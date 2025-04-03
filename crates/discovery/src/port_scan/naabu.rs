use anyhow::Result;
use serde_json::Value;
use std::net::IpAddr;
use std::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;

use crate::port_scan::DiscoveredPort;
use crate::results::DiscoveryResult;

pub struct NaabuRunner;

impl NaabuRunner {
    pub fn new() -> Self {
        NaabuRunner
    }

    pub async fn scan_target(
        &self,
        target: &str,
        ports: Option<&[u16]>,
    ) -> Result<DiscoveryResult> {
        let mut cmd = AsyncCommand::new("naabu");
        cmd.args(["-host", target, "-json", "-silent"]);

        if let Some(p) = ports {
            cmd.arg("-p");
            cmd.arg(
                p.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        } else {
            cmd.arg("-top-ports");
            cmd.arg("1000");
        }

        let mut child = cmd.stdout(std::process::Stdio::piped()).spawn()?;
        let stdout = child.stdout.take().expect("Failed to get stdout handle");
        let reader = BufReader::new(stdout).lines();

        let mut discovery_result = DiscoveryResult::new();
        let source = format!("naabu_scan_for_{}", target);

        // Process output line by line
        let mut reader = reader;
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(entry) = serde_json::from_str::<Value>(&line) {
                if let (Some(ip), Some(port)) = (
                    entry.get("ip").and_then(|i| i.as_str()),
                    entry.get("port").and_then(|p| p.as_u64()),
                ) {
                    if let Ok(ip_addr) = ip.parse::<IpAddr>() {
                        discovery_result.ports.push(DiscoveredPort {
                            ip_address: ip_addr,
                            port: port as u16,
                            protocol: "TCP".to_string(),
                            status: "OPEN".to_string(),
                            service_name: entry
                                .get("service")
                                .and_then(|s| s.as_str())
                                .map(String::from),
                            banner: None,
                            source: source.clone(),
                        });
                    }
                }
            }
        }

        // Wait for the child process to finish
        let status = child.wait().await?;
        if !status.success() {
            tracing::warn!("Naabu process exited with non-zero status: {}", status);
        }

        Ok(discovery_result)
    }

    pub async fn scan_target_sync(
        &self,
        target: &str,
        ports: Option<&[u16]>,
    ) -> Result<DiscoveryResult> {
        let mut cmd = Command::new("naabu");
        cmd.args(["-host", target, "-json", "-silent"]);

        if let Some(p) = ports {
            cmd.arg("-p");
            cmd.arg(
                p.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        } else {
            cmd.arg("-top-ports");
            cmd.arg("1000");
        }

        let output = cmd.output()?;
        let mut discovery_result = DiscoveryResult::new();
        let source = format!("naabu_scan_for_{}", target);

        // Parse JSON output line by line
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                if let (Some(ip), Some(port)) = (
                    entry.get("ip").and_then(|i| i.as_str()),
                    entry.get("port").and_then(|p| p.as_u64()),
                ) {
                    if let Ok(ip_addr) = ip.parse::<IpAddr>() {
                        discovery_result.ports.push(DiscoveredPort {
                            ip_address: ip_addr,
                            port: port as u16,
                            protocol: "TCP".to_string(),
                            status: "OPEN".to_string(),
                            service_name: entry
                                .get("service")
                                .and_then(|s| s.as_str())
                                .map(String::from),
                            banner: None,
                            source: source.clone(),
                        });
                    }
                }
            }
        }

        Ok(discovery_result)
    }
}
