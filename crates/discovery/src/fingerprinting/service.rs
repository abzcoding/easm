//! Service fingerprinting module
//!
//! This module implements detection of services running on open ports
//! beyond web technologies, such as:
//! - Database servers (MySQL, PostgreSQL, Redis, etc.)
//! - Mail servers (SMTP, IMAP, POP3)
//! - SSH servers
//! - FTP servers
//! - Other common network services

use crate::fingerprinting::Fingerprinter;
use crate::results::DiscoveryResult;
use shared::types::ID;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

/// Service Fingerprinter
pub struct ServiceFingerprinter {
    /// Timeout for connection attempts in seconds
    timeout_secs: u64,
    /// Signature database
    signatures: HashMap<u16, Vec<ServiceSignature>>,
}

/// Service signature for detection
struct ServiceSignature {
    /// Name of the service
    name: String,
    /// Service category
    category: String,
    /// Port number
    port: u16,
    /// Protocol (TCP/UDP)
    protocol: String,
    /// Banner to match (if any)
    banner_match: Option<String>,
    /// Probe data to send (if any)
    probe: Option<Vec<u8>>,
    /// Version extraction regex if available
    version_regex: Option<String>,
}

impl ServiceFingerprinter {
    /// Create a new ServiceFingerprinter
    pub fn new(timeout_secs: Option<u64>) -> Self {
        Self {
            timeout_secs: timeout_secs.unwrap_or(5),
            signatures: Self::load_signatures(),
        }
    }

    /// Load service signatures from embedded data
    fn load_signatures() -> HashMap<u16, Vec<ServiceSignature>> {
        let mut signatures = HashMap::new();

        // Database servers
        let db_signatures = vec![
            ServiceSignature {
                name: "MySQL".to_string(),
                category: "Database".to_string(),
                port: 3306,
                protocol: "TCP".to_string(),
                banner_match: Some("mysql".to_string()),
                probe: None,
                version_regex: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            ServiceSignature {
                name: "PostgreSQL".to_string(),
                category: "Database".to_string(),
                port: 5432,
                protocol: "TCP".to_string(),
                banner_match: None,
                probe: Some(vec![0x00, 0x00, 0x00, 0x08, 0x04, 0xd2, 0x16, 0x2f]),
                version_regex: None,
            },
            ServiceSignature {
                name: "Redis".to_string(),
                category: "Database".to_string(),
                port: 6379,
                protocol: "TCP".to_string(),
                banner_match: Some("redis".to_string()),
                probe: Some(b"INFO\r\n".to_vec()),
                version_regex: Some(r"redis_version:(\d+\.\d+\.\d+)".to_string()),
            },
        ];

        for sig in db_signatures {
            signatures
                .entry(sig.port)
                .or_insert_with(Vec::new)
                .push(sig);
        }

        // Mail servers
        let mail_signatures = vec![
            ServiceSignature {
                name: "SMTP".to_string(),
                category: "Mail".to_string(),
                port: 25,
                protocol: "TCP".to_string(),
                banner_match: Some("ESMTP".to_string()),
                probe: None,
                version_regex: None,
            },
            ServiceSignature {
                name: "IMAP".to_string(),
                category: "Mail".to_string(),
                port: 143,
                protocol: "TCP".to_string(),
                banner_match: Some("IMAP".to_string()),
                probe: None,
                version_regex: None,
            },
            ServiceSignature {
                name: "POP3".to_string(),
                category: "Mail".to_string(),
                port: 110,
                protocol: "TCP".to_string(),
                banner_match: Some("POP3".to_string()),
                probe: None,
                version_regex: None,
            },
        ];

        for sig in mail_signatures {
            signatures
                .entry(sig.port)
                .or_insert_with(Vec::new)
                .push(sig);
        }

        // SSH
        let ssh_signature = ServiceSignature {
            name: "SSH".to_string(),
            category: "Remote Access".to_string(),
            port: 22,
            protocol: "TCP".to_string(),
            banner_match: Some("SSH".to_string()),
            probe: None,
            version_regex: Some(r"SSH-\d+\.\d+-([^\s]+)".to_string()),
        };
        signatures
            .entry(22)
            .or_insert_with(Vec::new)
            .push(ssh_signature);

        // FTP
        let ftp_signature = ServiceSignature {
            name: "FTP".to_string(),
            category: "File Transfer".to_string(),
            port: 21,
            protocol: "TCP".to_string(),
            banner_match: Some("FTP".to_string()),
            probe: None,
            version_regex: None,
        };
        signatures
            .entry(21)
            .or_insert_with(Vec::new)
            .push(ftp_signature);

        signatures
    }

    /// Extract version from banner using regex pattern
    fn extract_version(&self, banner: &str, regex: &str) -> Option<String> {
        regex::Regex::new(regex)
            .ok()
            .and_then(|re| re.captures(banner))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Get the signature for a specific port
    fn get_signature_for_port(&self, port: u16) -> Option<&Vec<ServiceSignature>> {
        self.signatures.get(&port)
    }

    /// Fingerprint a service on a specific port
    async fn fingerprint_port(
        &self,
        host: &str,
        port: u16,
        asset_id: ID,
    ) -> Result<DiscoveryResult, anyhow::Error> {
        let mut result = DiscoveryResult::new();

        // Get signature for this port
        let signatures = match self.get_signature_for_port(port) {
            Some(sigs) => sigs,
            None => return Ok(result), // No signatures for this port
        };

        // Try to connect to the port
        let connect_future = TcpStream::connect(format!("{}:{}", host, port));
        let connect_timeout = Duration::from_secs(self.timeout_secs);

        let stream = match timeout(connect_timeout, connect_future).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                result.metadata.insert(
                    format!("port:{}_error", port),
                    format!("Connection error: {}", e),
                );
                return Ok(result);
            }
            Err(_) => {
                result.metadata.insert(
                    format!("port:{}_error", port),
                    "Connection timeout".to_string(),
                );
                return Ok(result);
            }
        };

        // Set read timeout
        let mut stream = stream;

        // Read initial banner if available
        let mut buffer = [0u8; 1024];
        let banner = match timeout(connect_timeout, stream.read(&mut buffer)).await {
            Ok(Ok(n)) if n > 0 => String::from_utf8_lossy(&buffer[..n]).to_string(),
            _ => String::new(),
        };

        if !banner.is_empty() {
            result
                .metadata
                .insert(format!("port:{}_banner", port), banner.clone());
        }

        // Check each signature for this port
        for signature in signatures {
            // If we have a banner, check for matches
            if let Some(banner_match) = &signature.banner_match {
                if !banner.is_empty()
                    && banner.to_lowercase().contains(&banner_match.to_lowercase())
                {
                    // Extract version if we have a regex
                    let version = signature
                        .version_regex
                        .as_ref()
                        .and_then(|re| self.extract_version(&banner, re));

                    // Add technology finding
                    result.technologies.push(crate::results::TechnologyFinding {
                        asset_id,
                        name: signature.name.clone(),
                        version,
                        category: Some(signature.category.clone()),
                        evidence: format!("Banner match on port {}: {}", port, banner),
                    });

                    continue; // Go to next signature
                }
            }

            // If we have a probe, send it and check the response
            if let Some(probe) = &signature.probe {
                // Try to write the probe
                if let Err(e) = stream.write_all(probe).await {
                    result.metadata.insert(
                        format!("port:{}_probe_error", port),
                        format!("Failed to send probe: {}", e),
                    );
                    continue;
                }

                // Read the response
                let mut buffer = [0u8; 1024];
                let response = match timeout(connect_timeout, stream.read(&mut buffer)).await {
                    Ok(Ok(n)) if n > 0 => String::from_utf8_lossy(&buffer[..n]).to_string(),
                    _ => String::new(),
                };

                if !response.is_empty() {
                    result
                        .metadata
                        .insert(format!("port:{}_probe_response", port), response.clone());

                    // Extract version if we have a regex
                    let version = signature
                        .version_regex
                        .as_ref()
                        .and_then(|re| self.extract_version(&response, re));

                    // Add technology finding
                    result.technologies.push(crate::results::TechnologyFinding {
                        asset_id,
                        name: signature.name.clone(),
                        version,
                        category: Some(signature.category.clone()),
                        evidence: format!("Probe response on port {}", port),
                    });
                }
            }
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl Fingerprinter for ServiceFingerprinter {
    async fn fingerprint(&self, target: &str, asset_id: ID) -> DiscoveryResult {
        let mut result = DiscoveryResult::new();

        // Common ports to scan
        let common_ports = vec![21, 22, 25, 80, 110, 143, 443, 3306, 5432, 6379, 8080];

        for port in common_ports {
            match self.fingerprint_port(target, port, asset_id).await {
                Ok(port_result) => {
                    // Merge results
                    result.technologies.extend(port_result.technologies);
                    result.metadata.extend(port_result.metadata);
                }
                Err(e) => {
                    result.metadata.insert(
                        format!("port:{}_error", port),
                        format!("Error fingerprinting: {}", e),
                    );
                }
            }
        }

        result
    }
}
