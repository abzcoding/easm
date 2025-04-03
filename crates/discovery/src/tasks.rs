use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryTaskType {
    DnsEnumeration,
    PortScan,
    PortScanNaabu, // Add naabu-specific task type
    WebAppScan,
    WebAppScanHttpx, // Add httpx-specific task type
    CertificateTransparency,
    VulnerabilityScanNuclei, // Add Nuclei vulnerability scanning task type
                             // Add other task types as needed
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NucleiTaskParams {
    pub templates: Option<Vec<String>>, // Specific templates to use
    pub severity: Option<String>,       // Filter by severity (critical,high,medium,low,info)
    pub rate_limit: Option<u32>,        // Rate limit for requests per second
    pub follow_redirects: Option<bool>, // Whether to follow redirects
    pub max_host_error: Option<u32>,    // Maximum errors for a host before skipping
    pub timeout: Option<u32>,           // Timeout in seconds for each template execution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTask {
    pub job_id: Uuid,
    pub organization_id: Uuid,
    pub task_type: DiscoveryTaskType,
    pub target: String,                          // e.g., domain name, IP range
    pub nuclei_params: Option<NucleiTaskParams>, // Parameters for Nuclei tasks
}

// Implement method to execute tasks
impl DiscoveryTask {
    pub async fn execute(&self) -> anyhow::Result<crate::results::DiscoveryResult> {
        match self.task_type {
            DiscoveryTaskType::PortScanNaabu => {
                let scanner = crate::port_scan::naabu::NaabuRunner::new();
                scanner.scan_target(&self.target, None).await
            }
            DiscoveryTaskType::WebAppScanHttpx => {
                let scanner = crate::web_crawl::httpx::HttpxRunner::new();
                scanner.scan_urls(&[self.target.clone()]).await
            }
            DiscoveryTaskType::VulnerabilityScanNuclei => {
                let mut scanner = crate::vulnerability::nuclei::NucleiRunner::new();

                // Apply Nuclei-specific parameters if available
                if let Some(params) = &self.nuclei_params {
                    if let Some(templates) = &params.templates {
                        scanner = scanner.with_templates(templates.clone());
                    }

                    if let Some(severity) = &params.severity {
                        scanner = scanner.with_severity(severity.clone());
                    }

                    if let Some(rate) = params.rate_limit {
                        scanner = scanner.with_rate_limit(rate);
                    }
                }

                scanner.scan_targets(&[self.target.clone()]).await
            }
            // Handle other task types with default implementations
            DiscoveryTaskType::PortScan => {
                // Use the built-in scanner
                crate::port_scan::scan_ip(self.target.parse()?, &[80, 443, 8080]).await
            }
            DiscoveryTaskType::WebAppScan => {
                // Use the built-in crawler
                crate::web_crawl::crawl_url(&self.target, 1).await
            }
            DiscoveryTaskType::DnsEnumeration => {
                // Use the built-in DNS enumerator
                let results = crate::dns::enumerate_domain(&self.target).await?;
                // Take the first result or create a new empty one if no results
                Ok(results
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| crate::results::DiscoveryResult::new()))
            }
            DiscoveryTaskType::CertificateTransparency => {
                // Not implemented yet
                anyhow::bail!("Certificate transparency tasks not yet implemented")
            }
        }
    }
}
