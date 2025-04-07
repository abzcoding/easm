use crate::results::{DiscoveredIp, DiscoveryResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, timeout};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredPort {
    pub ip_address: IpAddr,
    pub port: u16,
    pub protocol: String, // "TCP" or "UDP"
    pub status: String,   // "OPEN", "CLOSED", "FILTERED"
    pub service_name: Option<String>,
    pub banner: Option<String>,
    pub source: String,
}

const TCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const UDP_PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const BANNER_GRAB_TIMEOUT: Duration = Duration::from_secs(5);

// Common service to port mappings
lazy_static::lazy_static! {
    static ref SERVICE_PORTS: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(21, "FTP");
        m.insert(22, "SSH");
        m.insert(23, "Telnet");
        m.insert(25, "SMTP");
        m.insert(53, "DNS");
        m.insert(80, "HTTP");
        m.insert(110, "POP3");
        m.insert(123, "NTP");
        m.insert(143, "IMAP");
        m.insert(443, "HTTPS");
        m.insert(465, "SMTPS");
        m.insert(587, "Submission");
        m.insert(993, "IMAPS");
        m.insert(995, "POP3S");
        m.insert(1433, "MSSQL");
        m.insert(1521, "Oracle");
        m.insert(3306, "MySQL");
        m.insert(3389, "RDP");
        m.insert(5432, "PostgreSQL");
        m.insert(5900, "VNC");
        m.insert(6379, "Redis");
        m.insert(8080, "HTTP-Alt");
        m.insert(8443, "HTTPS-Alt");
        m.insert(9200, "Elasticsearch");
        m.insert(27017, "MongoDB");
        m
    };
}

pub async fn scan_ip(target_ip: IpAddr, ports: &[u16]) -> Result<DiscoveryResult> {
    tracing::debug!("Scanning IP: {} for {} ports", target_ip, ports.len());

    let (tx, mut rx) = mpsc::channel::<DiscoveredPort>(ports.len() * 2); // Channel for port results
    let source_base = format!("port_scan_for_{}", target_ip);

    // Use a semaphore to limit concurrent scans
    let max_concurrent = 100;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));

    // Track open TCP ports for banner grabbing
    let open_tcp_ports = Arc::new(Mutex::new(Vec::new()));

    // TCP scan
    for &port in ports {
        let tx_clone = tx.clone();
        let source = source_base.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        let open_ports = open_tcp_ports.clone();

        tokio::spawn(async move {
            let _permit = permit; // Drop at end of scope
            let tcp_result = scan_tcp_port(target_ip, port, source.clone()).await;

            if let Some(port_info) = tcp_result {
                // If port is open, add to open ports list for banner grabbing
                if port_info.status == "OPEN" {
                    open_ports.lock().await.push(port);
                }

                if tx_clone.send(port_info).await.is_err() {
                    tracing::error!(
                        "Failed to send TCP port scan result for {}:{}",
                        target_ip,
                        port
                    );
                }
            }
        });
    }

    // UDP scan (can be slower and less reliable)
    if !ports.is_empty() {
        let udp_ports = ports
            .iter()
            .cloned()
            .filter(|&p| p != 80 && p != 443) // Skip common web ports for UDP
            .collect::<Vec<_>>();

        for &port in &udp_ports {
            let tx_clone = tx.clone();
            let source = source_base.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            tokio::spawn(async move {
                let _permit = permit; // Drop at end of scope
                let udp_result = scan_udp_port(target_ip, port, source).await;

                if let Some(port_info) = udp_result {
                    if tx_clone.send(port_info).await.is_err() {
                        tracing::error!(
                            "Failed to send UDP port scan result for {}:{}",
                            target_ip,
                            port
                        );
                    }
                }
            });
        }
    }

    // Wait a bit for initial scans to complete
    sleep(Duration::from_millis(100)).await;

    // Drop the original sender to close the channel when all tasks are done
    drop(tx);

    // Create a single DiscoveryResult to hold all findings
    let mut discovery_result = DiscoveryResult::new();

    // Add the target IP to the discovery result
    discovery_result.ip_addresses.push(DiscoveredIp {
        ip_address: target_ip,
        source: format!("port_scan_target_{}", target_ip),
    });

    // Perform banner grabbing for open TCP ports
    let open_ports = open_tcp_ports.lock().await.clone();
    if !open_ports.is_empty() {
        tracing::debug!(
            "Attempting banner grabbing on {} open ports",
            open_ports.len()
        );

        let banner_results = grab_banners(target_ip, &open_ports, &source_base).await?;

        // Collect ports and update with banner information
        while let Some(mut port_info) = rx.recv().await {
            // If we have banner info for this port, add it
            if let Some(banner_data) = banner_results.get(&port_info.port) {
                port_info.banner = Some(banner_data.banner.clone());

                // If we detected a service from the banner, use it
                if !banner_data.detected_service.is_empty() {
                    port_info.service_name = Some(banner_data.detected_service.clone());
                }
            }
            discovery_result.ports.push(port_info);
        }
    } else {
        // No open ports for banner grabbing, just collect the scan results
        while let Some(port_info) = rx.recv().await {
            discovery_result.ports.push(port_info);
        }
    }

    Ok(discovery_result)
}

async fn scan_tcp_port(ip: IpAddr, port: u16, source: String) -> Option<DiscoveredPort> {
    let addr: std::net::SocketAddr = (ip, port).into();
    let result = timeout(TCP_CONNECT_TIMEOUT, TcpStream::connect(addr)).await;

    match result {
        Ok(Ok(_stream)) => {
            // Connection successful - Port is OPEN
            let service = SERVICE_PORTS.get(&port).map(|s| s.to_string());

            Some(DiscoveredPort {
                ip_address: ip,
                port,
                protocol: "TCP".to_string(),
                status: "OPEN".to_string(),
                service_name: service,
                banner: None, // Will be filled later if banner grabbing succeeds
                source,
            })
        }
        Ok(Err(_)) => {
            // Connection refused - Port is CLOSED
            Some(DiscoveredPort {
                ip_address: ip,
                port,
                protocol: "TCP".to_string(),
                status: "CLOSED".to_string(),
                service_name: None,
                banner: None,
                source,
            })
        }
        Err(_) => {
            // Timeout - Port is FILTERED
            Some(DiscoveredPort {
                ip_address: ip,
                port,
                protocol: "TCP".to_string(),
                status: "FILTERED".to_string(),
                service_name: None,
                banner: None,
                source,
            })
        }
    }
}

async fn scan_udp_port(ip: IpAddr, port: u16, source: String) -> Option<DiscoveredPort> {
    // UDP scanning is trickier - sending empty packet and checking for ICMP response
    // This is a simplified version that may have false positives/negatives
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to bind UDP socket: {}", e);
            return None;
        }
    };

    let addr: std::net::SocketAddr = (ip, port).into();
    // Send empty UDP packet
    if socket.send_to(&[0; 10], addr).await.is_err() {
        return Some(DiscoveredPort {
            ip_address: ip,
            port,
            protocol: "UDP".to_string(),
            status: "ERROR".to_string(),
            service_name: None,
            banner: None,
            source,
        });
    }

    // Try to receive a response
    let mut buf = [0; 65535];
    let recv_result = timeout(UDP_PROBE_TIMEOUT, socket.recv(&mut buf)).await;

    let status = match recv_result {
        Ok(Ok(_)) => "OPEN",       // Got a response, port might be open
        Ok(Err(_)) => "ERROR",     // Error receiving
        Err(_) => "OPEN|FILTERED", // No response, could be filtered or open
    };

    let service = if status == "OPEN" {
        SERVICE_PORTS.get(&port).map(|s| s.to_string())
    } else {
        None
    };

    Some(DiscoveredPort {
        ip_address: ip,
        port,
        protocol: "UDP".to_string(),
        status: status.to_string(),
        service_name: service,
        banner: None,
        source,
    })
}

struct BannerResult {
    #[allow(dead_code)]
    port: u16,
    banner: String,
    detected_service: String,
}

async fn grab_banners(
    ip: IpAddr,
    ports: &[u16],
    _source_base: &str,
) -> Result<HashMap<u16, BannerResult>> {
    let mut results: HashMap<u16, BannerResult> = HashMap::new();

    for &port in ports {
        match timeout(
            BANNER_GRAB_TIMEOUT,
            grab_banner_for_port(ip, port, _source_base),
        )
        .await
        {
            Ok(Ok(Some(banner_result))) => {
                results.insert(port, banner_result);
            }
            Ok(Ok(None)) => {} // No banner grabbed
            Ok(Err(e)) => {
                tracing::debug!("Error grabbing banner for {}:{}: {}", ip, port, e);
            }
            Err(_) => {
                tracing::debug!("Timeout grabbing banner for {}:{}", ip, port);
            }
        }
    }

    Ok(results)
}

async fn grab_banner_for_port(
    ip: IpAddr,
    port: u16,
    _source_base: &str,
) -> Result<Option<BannerResult>> {
    let addr: std::net::SocketAddr = (ip, port).into();
    let mut stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("Could not connect for banner grabbing: {}", e);
            return Ok(None);
        }
    };

    // Some services need a stimulus packet to respond
    let stimulus = match port {
        21 => b"USER anonymous\r\n".to_vec(),                // FTP
        25 | 587 | 465 => b"EHLO easm.scanner\r\n".to_vec(), // SMTP
        80 | 8080 => b"GET / HTTP/1.0\r\nHost: host\r\n\r\n".to_vec(), // HTTP
        110 => b"CAPA\r\n".to_vec(),                         // POP3
        143 => b"A001 CAPABILITY\r\n".to_vec(),              // IMAP
        _ => Vec::new(),
    };

    if !stimulus.is_empty() {
        // If error sending stimulus, try without it
        let _ = tokio::io::AsyncWriteExt::write(&mut stream, &stimulus).await;
    }

    // Read response
    let mut buf = vec![0; 4096];
    let n = match timeout(
        Duration::from_secs(2),
        tokio::io::AsyncReadExt::read(&mut stream, &mut buf),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => n,
        _ => return Ok(None),
    };

    // Try to interpret as UTF-8, fall back to lossy if it's not valid
    let banner_text = String::from_utf8_lossy(&buf[..n]).to_string();
    let banner_clean = clean_banner(&banner_text);

    // Attempt to detect service from banner
    let detected_service = detect_service_from_banner(&banner_clean, port);

    Ok(Some(BannerResult {
        port,
        banner: banner_clean,
        detected_service,
    }))
}

fn clean_banner(banner: &str) -> String {
    // Remove non-printable characters, limit length, etc.
    let mut cleaned = banner
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect::<String>();

    // Limit length
    if cleaned.len() > 1024 {
        cleaned.truncate(1024);
        cleaned.push_str("...");
    }

    cleaned
}

fn detect_service_from_banner(banner: &str, port: u16) -> String {
    let banner_lower = banner.to_lowercase();

    // Look for common service signatures in banner
    if banner_lower.contains("ssh") {
        return "SSH".to_string();
    } else if banner_lower.contains("ftp") {
        return "FTP".to_string();
    } else if banner_lower.contains("smtp") || banner_lower.contains("mail service") {
        return "SMTP".to_string();
    } else if banner_lower.contains("http") {
        if port == 443 || banner_lower.contains("ssl") || banner_lower.contains("https") {
            return "HTTPS".to_string();
        }
        return "HTTP".to_string();
    } else if banner_lower.contains("pop3") {
        return "POP3".to_string();
    } else if banner_lower.contains("imap") {
        return "IMAP".to_string();
    } else if banner_lower.contains("mysql") {
        return "MySQL".to_string();
    } else if banner_lower.contains("postgresql") {
        return "PostgreSQL".to_string();
    } else if banner_lower.contains("mongodb") {
        return "MongoDB".to_string();
    } else if banner_lower.contains("redis") {
        return "Redis".to_string();
    } else if banner_lower.contains("vnc") {
        return "VNC".to_string();
    } else if banner_lower.contains("rdp") || banner_lower.contains("remote desktop") {
        return "RDP".to_string();
    }

    // Fall back to known port associations
    SERVICE_PORTS
        .get(&port)
        .map_or(String::new(), |s| s.to_string())
}

// Add the naabu module
pub mod naabu;

/// Port Scanner struct for scanning IP addresses
pub struct PortScanner {}

impl Default for PortScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PortScanner {
    /// Create a new port scanner
    pub fn new() -> Self {
        Self {}
    }

    /// Scan an IP address for open ports
    /// If ports is None, scans common ports
    pub async fn scan_ip(&self, ip_str: &str, ports: Option<&[u16]>) -> Result<DiscoveryResult> {
        // Parse IP
        let ip = match ip_str.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(e) => {
                return Err(anyhow::anyhow!("Invalid IP address {}: {}", ip_str, e));
            }
        };

        // Use common ports if none specified
        let ports_to_scan = match ports {
            Some(p) => p.to_vec(),
            None => vec![
                21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995, 1723, 3306,
                3389, 5900, 8080, 8443,
            ],
        };

        // Scan the IP
        scan_ip(ip, &ports_to_scan).await
    }
}
