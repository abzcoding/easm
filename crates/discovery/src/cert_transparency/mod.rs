use crate::results::{DiscoveredDomain, DiscoveryResult};
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct CrtShEntry {
    #[serde(rename = "common_name")]
    common_name: Option<String>,
    #[serde(rename = "name_value")]
    name_value: Option<String>,
}

pub async fn monitor_logs(domain: &str) -> Result<DiscoveryResult> {
    tracing::debug!("Monitoring Certificate Transparency logs for: {}", domain);
    let client = Client::builder()
        .user_agent("EASM Discovery Bot/0.1")
        .timeout(std::time::Duration::from_secs(30)) // CT logs can be slow
        .build()?;

    let url = format!("https://crt.sh/?q={}&output=json", domain);
    let mut discovery_result = DiscoveryResult::new();
    let mut found_domains: HashSet<String> = HashSet::new();
    let source = format!("crt.sh_for_{}", domain);

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Vec<CrtShEntry>>().await {
                    Ok(entries) => {
                        tracing::trace!(
                            "Received {} entries from crt.sh for {}",
                            entries.len(),
                            domain
                        );
                        for entry in entries {
                            // Extract domains from common_name and name_value
                            if let Some(cn) = entry.common_name {
                                process_potential_domain(&cn, &mut found_domains);
                            }
                            if let Some(names) = entry.name_value {
                                for name in names.split("\\n") {
                                    process_potential_domain(name, &mut found_domains);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse JSON from crt.sh for {}: {}", domain, e);
                        // Handle cases where crt.sh returns non-JSON or malformed JSON
                        // Often happens if no results are found, crt.sh might return HTML.
                    }
                }
            } else {
                tracing::warn!(
                    "crt.sh query for {} failed with status: {}",
                    domain,
                    response.status()
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to query crt.sh for {}: {}", domain, e);
        }
    }

    // Convert unique domains found into DiscoveryResult
    for found_domain in found_domains {
        // Basic filtering: avoid wildcards for now, ensure it looks like a domain
        if !found_domain.starts_with("*.") && found_domain.contains('.') {
            discovery_result.domains.push(DiscoveredDomain {
                domain_name: found_domain,
                source: source.clone(),
            });
        }
    }

    Ok(discovery_result)
}

// Helper to clean up and potentially add a domain string to the set
fn process_potential_domain(name: &str, found_domains: &mut HashSet<String>) {
    let cleaned_name = name.trim();
    // Add more validation/cleaning if needed
    if !cleaned_name.is_empty() {
        found_domains.insert(cleaned_name.to_lowercase());
    }
}
