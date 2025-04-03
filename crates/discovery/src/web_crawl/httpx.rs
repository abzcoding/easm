use anyhow::Result;
use serde_json::Value;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;

use crate::results::{DiscoveredWebResource, DiscoveryResult};

pub struct HttpxRunner;

impl HttpxRunner {
    pub fn new() -> Self {
        HttpxRunner
    }

    pub async fn scan_urls(&self, urls: &[String]) -> Result<DiscoveryResult> {
        // Write URLs to a temporary file
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", urls.join("\n"))?;

        let mut cmd = AsyncCommand::new("httpx");
        cmd.args([
            "-l",
            temp_file.path().to_str().unwrap(),
            "-json",
            "-silent",
            "-tech-detect",
            "-title",
            "-status-code",
            "-response-time",
            "-server",
        ]);

        let mut child = cmd.stdout(std::process::Stdio::piped()).spawn()?;
        let stdout = child.stdout.take().expect("Failed to get stdout handle");
        let reader = BufReader::new(stdout).lines();

        let mut discovery_result = DiscoveryResult::new();

        // Process output line by line
        let mut reader = reader;
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(entry) = serde_json::from_str::<Value>(&line) {
                if let Some(url) = entry.get("url").and_then(|u| u.as_str()) {
                    let source = format!("httpx_scan_for_{}", url);

                    let technologies = entry
                        .get("tech")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    discovery_result.web_resources.push(DiscoveredWebResource {
                        url: url.to_string(),
                        status_code: entry
                            .get("status_code")
                            .and_then(|c| c.as_u64())
                            .unwrap_or(0) as u16,
                        title: entry
                            .get("title")
                            .and_then(|t| t.as_str())
                            .map(String::from),
                        technologies,
                        source,
                    });
                }
            }
        }

        // Wait for the child process to finish
        let status = child.wait().await?;
        if !status.success() {
            tracing::warn!("HTTPx process exited with non-zero status: {}", status);
        }

        Ok(discovery_result)
    }

    pub fn scan_urls_sync(&self, urls: &[String]) -> Result<DiscoveryResult> {
        // Write URLs to a temporary file
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", urls.join("\n"))?;

        let output = Command::new("httpx")
            .args([
                "-l",
                temp_file.path().to_str().unwrap(),
                "-json",
                "-silent",
                "-tech-detect",
                "-title",
                "-status-code",
                "-response-time",
                "-server",
            ])
            .output()?;

        let mut discovery_result = DiscoveryResult::new();

        // Parse JSON output line by line
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                if let Some(url) = entry.get("url").and_then(|u| u.as_str()) {
                    let source = format!("httpx_scan_for_{}", url);

                    let technologies = entry
                        .get("tech")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    discovery_result.web_resources.push(DiscoveredWebResource {
                        url: url.to_string(),
                        status_code: entry
                            .get("status_code")
                            .and_then(|c| c.as_u64())
                            .unwrap_or(0) as u16,
                        title: entry
                            .get("title")
                            .and_then(|t| t.as_str())
                            .map(String::from),
                        technologies,
                        source,
                    });
                }
            }
        }

        Ok(discovery_result)
    }
}

impl Default for HttpxRunner {
    fn default() -> Self {
        Self::new()
    }
}
