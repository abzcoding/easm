// Placeholder for DNS Discovery logic

// Consider using trust-dns-resolver crate
// use trust_dns_resolver::TokioAsyncResolver;

use crate::results::{DiscoveredDomain, DiscoveredIp, DiscoveryResult};
use anyhow::Result;
use lazy_static::lazy_static;
use std::sync::Mutex;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::TokioAsyncResolver as dnsresolv;

// Create a lazily initialized global resolver
lazy_static! {
    static ref DNS_RESOLVER: Mutex<Option<TokioAsyncResolver>> = Mutex::new(None);
}

// Helper function to get or initialize the resolver
async fn get_resolver() -> Result<TokioAsyncResolver> {
    let mut resolver_guard = DNS_RESOLVER
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire DNS resolver lock: {}", e))?;

    if resolver_guard.is_none() {
        // Initialize the resolver if not already done
        tracing::debug!("Initializing DNS resolver");

        // This isn't actually fallible in the same way as other Result-returning functions
        *resolver_guard = Some(TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        ));
    }

    // Clone the resolver for the caller
    Ok(resolver_guard.as_ref().unwrap().clone())
}

pub async fn enumerate_domain(target_domain: &str) -> Result<Vec<DiscoveryResult>> {
    tracing::debug!("Enumerating domain: {}", target_domain);
    // Get the shared resolver instead of creating a new one each time
    let resolver = get_resolver().await?;

    let mut results = Vec::new();
    let source = format!("dns_enum_for_{}", target_domain);

    // Create a single DiscoveryResult to accumulate all findings
    let mut discovery_result = DiscoveryResult::new();

    // A / AAAA Records (IPv4 and IPv6)
    match resolver.lookup_ip(target_domain).await {
        Ok(response) => {
            for ip in response.iter() {
                tracing::trace!("Found IP: {} for {}", ip, target_domain);
                discovery_result.ip_addresses.push(DiscoveredIp {
                    ip_address: ip,
                    source: source.clone(),
                });
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to lookup A/AAAA records for {}: {}",
                target_domain,
                e
            );
        }
    }

    // CNAME Record
    match resolver.lookup(target_domain, RecordType::CNAME).await {
        Ok(response) => {
            for record in response.iter() {
                if let Some(cname) = record.as_cname() {
                    let domain_name = cname.to_utf8();
                    tracing::trace!("Found CNAME: {} -> {}", target_domain, domain_name);
                    discovery_result.domains.push(DiscoveredDomain {
                        domain_name: domain_name.clone(),
                        source: source.clone(),
                    });
                    // Optionally recurse
                    // let subdomain_results = enumerate_domain(&domain_name).await?;
                    // discovery_result.merge(subdomain_results[0].clone());
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to lookup CNAME record for {}: {}", target_domain, e);
        }
    }

    // MX Record
    match resolver.lookup(target_domain, RecordType::MX).await {
        Ok(response) => {
            for record in response.iter() {
                if let Some(mx) = record.as_mx() {
                    let exchange_domain = mx.exchange().to_utf8();
                    tracing::trace!(
                        "Found MX: {} -> {} (preference: {})",
                        target_domain,
                        &exchange_domain,
                        mx.preference()
                    );
                    // Add the exchange domain as a discovered asset
                    discovery_result.domains.push(DiscoveredDomain {
                        domain_name: exchange_domain,
                        source: source.clone(),
                    });
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to lookup MX record for {}: {}", target_domain, e);
        }
    }

    // TXT Record
    match resolver.lookup(target_domain, RecordType::TXT).await {
        Ok(response) => {
            for record in response.iter() {
                if let Some(txt) = record.as_txt() {
                    for txt_part in txt.iter() {
                        if let Ok(txt_string) = std::str::from_utf8(txt_part) {
                            tracing::trace!("Found TXT: {} -> \"{}\"", target_domain, txt_string);
                            // TODO: Implement parsing for common TXT record formats (SPF, DKIM, DMARC, etc.)
                            //       Extract potential domains, IPs, or other relevant info.
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to lookup TXT record for {}: {}", target_domain, e);
        }
    }

    // Add the populated result to our list of results
    results.push(discovery_result);

    Ok(results)
}

// Standalone function for resolving domains to IPs
pub async fn resolve_domain(domain: &str) -> Vec<std::net::IpAddr> {
    let resolver = match get_resolver().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to get resolver: {}", e);
            return Vec::new();
        }
    };

    match resolver.lookup_ip(domain).await {
        Ok(ips) => ips.iter().collect(),
        Err(e) => {
            tracing::warn!("Failed to resolve domain {}: {}", domain, e);
            Vec::new()
        }
    }
}

pub struct DnsEnumerator {
    resolver: dnsresolv,
}

impl DnsEnumerator {
    pub async fn new() -> Result<Self> {
        // The tokio function returns a resolver directly, not a Result
        let resolver = dnsresolv::tokio(ResolverConfig::default(), ResolverOpts::default());

        Ok(DnsEnumerator { resolver })
    }

    /// Resolve domain to IP addresses
    pub async fn resolve(&self, domain: &str) -> Result<Vec<std::net::IpAddr>> {
        let ips = self
            .resolver
            .lookup_ip(domain)
            .await
            .map_err(|e| anyhow::anyhow!("DNS lookup failed for {}: {}", domain, e))?;

        let mut results = Vec::new();
        for ip in ips.iter() {
            results.push(ip);
        }

        Ok(results)
    }

    /// Perform DNS enumeration on a domain
    pub async fn enumerate(&self, domain: &str) -> Result<DiscoveryResult> {
        let mut result = DiscoveryResult::new();
        let source = format!("dns_enum_for_{}", domain);

        // A/AAAA records
        if let Ok(ips) = self.resolver.lookup_ip(domain).await {
            for ip in ips.iter() {
                result.ip_addresses.push(DiscoveredIp {
                    ip_address: ip,
                    source: source.clone(),
                });
            }
        }

        // Add the domain itself
        result.domains.push(DiscoveredDomain {
            domain_name: domain.to_string(),
            source: "dns_input".to_string(),
        });

        // CNAME records
        if let Ok(cname_lookup) = self.resolver.lookup(domain, RecordType::CNAME).await {
            for record in cname_lookup.iter() {
                if let Some(cname) = record.as_cname() {
                    let cname_str = cname.to_utf8();
                    result.domains.push(DiscoveredDomain {
                        domain_name: cname_str,
                        source: "dns_cname".to_string(),
                    });
                }
            }
        }

        // MX records
        if let Ok(mx_lookup) = self.resolver.lookup(domain, RecordType::MX).await {
            for record in mx_lookup.iter() {
                if let Some(mx) = record.as_mx() {
                    let mx_str = mx.exchange().to_utf8();
                    result.domains.push(DiscoveredDomain {
                        domain_name: mx_str,
                        source: "dns_mx".to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Perform DNS brute-force enumeration
    pub async fn brute_force(&self, domain: &str, wordlist: &[String]) -> Result<DiscoveryResult> {
        let mut result = DiscoveryResult::new();

        for word in wordlist {
            let subdomain = format!("{}.{}", word, domain);

            if let Ok(ips) = self.resolver.lookup_ip(&subdomain).await {
                if ips.iter().next().is_some() {
                    result.domains.push(DiscoveredDomain {
                        domain_name: subdomain.clone(),
                        source: "dns_brute_force".to_string(),
                    });

                    // Add IP addresses
                    for ip in ips.iter() {
                        result.ip_addresses.push(DiscoveredIp {
                            ip_address: ip,
                            source: format!("dns_brute_force_for_{}", subdomain),
                        });
                    }
                }
            }
        }

        Ok(result)
    }
}
