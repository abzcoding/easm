use crate::port_scan::DiscoveredPort;
use crate::vulnerability::DiscoveredVulnerability;
use serde::{Deserialize, Serialize};
use shared::types::ID;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DiscoveredIp {
    pub ip_address: IpAddr,
    pub source: String, // e.g., "dns_lookup", "port_scan_target"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DiscoveredDomain {
    pub domain_name: String,
    pub source: String, // e.g., "certificate_transparency", "dns_enum"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredWebResource {
    pub url: String,
    pub status_code: u16,
    pub title: Option<String>,
    pub technologies: Vec<String>, // e.g., ["React", "Nginx"]
    pub source: String,
}

/// Technology finding that can be added to an asset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnologyFinding {
    /// ID of the asset this technology belongs to
    pub asset_id: ID,
    /// Name of the technology (e.g., "Apache", "WordPress")
    pub name: String,
    /// Detected version
    pub version: Option<String>,
    /// Technology category (e.g., "Web Server", "CMS")
    pub category: Option<String>,
    /// Evidence of the detection
    pub evidence: String,
}

/// Vulnerability finding that can be added to an asset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VulnerabilityFinding {
    /// ID of the asset this vulnerability was found on
    pub asset_id: ID,
    /// Optional port ID if the vulnerability is related to a specific port
    pub port_id: Option<ID>,
    /// Title of the vulnerability
    pub title: String,
    /// Detailed description
    pub description: Option<String>,
    /// CVE ID if applicable
    pub cve_id: Option<String>,
    /// Evidence of the vulnerability
    pub evidence: String,
    /// Severity level (as string, will be converted to enum)
    pub severity: String,
    /// Recommended remediation steps
    pub remediation: Option<String>,
}

/// Consolidated discovery result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryResult {
    /// Discovered IP addresses
    pub ip_addresses: Vec<DiscoveredIp>,
    /// Discovered domains
    pub domains: Vec<DiscoveredDomain>,
    /// Discovered ports
    pub ports: Vec<DiscoveredPort>,
    /// Discovered web resources
    pub web_resources: Vec<DiscoveredWebResource>,
    /// Discovered technologies
    pub technologies: Vec<TechnologyFinding>,
    /// Discovered vulnerabilities (for database storage)
    pub vulnerabilities: Vec<VulnerabilityFinding>,
    /// Raw vulnerability findings from scanners like Nuclei
    pub raw_vulnerabilities: Vec<DiscoveredVulnerability>,
    /// Additional metadata from the discovery process
    pub metadata: HashMap<String, String>,
}

impl DiscoveryResult {
    /// Create a new empty discovery result
    pub fn new() -> Self {
        Self {
            ip_addresses: Vec::new(),
            domains: Vec::new(),
            ports: Vec::new(),
            web_resources: Vec::new(),
            technologies: Vec::new(),
            vulnerabilities: Vec::new(),
            raw_vulnerabilities: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Merge another discovery result into this one
    pub fn merge(&mut self, other: DiscoveryResult) {
        self.ip_addresses.extend(other.ip_addresses);
        self.domains.extend(other.domains);
        self.ports.extend(other.ports);
        self.web_resources.extend(other.web_resources);
        self.technologies.extend(other.technologies);
        self.vulnerabilities.extend(other.vulnerabilities);
        self.raw_vulnerabilities.extend(other.raw_vulnerabilities);
        self.metadata.extend(other.metadata);
    }

    /// Convert raw vulnerabilities to VulnerabilityFindings
    pub fn convert_raw_vulnerabilities(&mut self, asset_id: ID) {
        for raw_vuln in &self.raw_vulnerabilities {
            self.vulnerabilities.push(VulnerabilityFinding {
                asset_id,
                port_id: None, // Would need additional logic to match to a port
                title: raw_vuln.name.clone(),
                description: raw_vuln.description.clone(),
                cve_id: raw_vuln.cve_id.clone(),
                evidence: format!(
                    "Found at: {}. Source: {}",
                    raw_vuln.matched_at, raw_vuln.source
                ),
                severity: raw_vuln.severity.clone(),
                remediation: None, // Nuclei doesn't consistently provide remediation
            });
        }
    }
}
