use crate::port_scan::DiscoveredPort;
use serde::{Deserialize, Serialize};
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

// A general enum to hold different types of discovery results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryResult {
    Ip(DiscoveredIp),
    Domain(DiscoveredDomain),
    Port(DiscoveredPort),
    WebResource(DiscoveredWebResource),
    // Add other result types here as needed (e.g., Technology, Vulnerability)
}
