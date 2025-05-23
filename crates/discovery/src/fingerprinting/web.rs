//! Web technology fingerprinting module
//!
//! This module implements detection of web technologies such as:
//! - Web servers (Apache, Nginx, IIS, etc.)
//! - Web frameworks (React, Angular, Vue, etc.)
//! - Content Management Systems (WordPress, Drupal, etc.)
//! - JavaScript libraries
//! - Analytics tools
//! - Security headers

use crate::fingerprinting::Fingerprinter;
use crate::results::{DiscoveryResult, TechnologyFinding};
use reqwest::header::HeaderMap;
use reqwest::Client;
use shared::types::ID;
use std::collections::HashMap;
use std::time::Duration;

/// Web Technology Fingerprinter
pub struct WebFingerprinter {
    client: Client,
    /// Signature database for technology detection
    signatures: HashMap<String, Vec<TechSignature>>,
}

/// Technology signature for detection
struct TechSignature {
    /// Name of the technology
    name: String,
    /// Technology category
    category: String,
    /// Detection method
    method: DetectionMethod,
    /// Pattern to match
    pattern: String,
    /// Version extraction regex if available
    version_regex: Option<String>,
}

/// Method used to detect technology
enum DetectionMethod {
    /// Match in HTTP headers
    Header(String),
    /// Match in HTML content
    Content,
    /// Match in JavaScript variables
    Script,
    /// Match in cookies
    Cookie(String),
    /// Match in URL patterns
    Url,
}

impl WebFingerprinter {
    /// Create a new WebFingerprinter with default signatures
    pub fn new() -> Result<Self, anyhow::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("EASM-Scanner/1.0")
            .build()?;

        // Load basic signatures for common technologies
        let signatures = Self::load_signatures();

        Ok(Self { client, signatures })
    }

    /// Load technology signatures from embedded data
    fn load_signatures() -> HashMap<String, Vec<TechSignature>> {
        let mut signatures = HashMap::new();

        // Web servers
        let web_servers = vec![
            TechSignature {
                name: "Apache".to_string(),
                category: "Web Server".to_string(),
                method: DetectionMethod::Header("Server".to_string()),
                pattern: "Apache".to_string(),
                version_regex: Some(r"Apache/(\d+[\.\d]*)".to_string()),
            },
            TechSignature {
                name: "Nginx".to_string(),
                category: "Web Server".to_string(),
                method: DetectionMethod::Header("Server".to_string()),
                pattern: "nginx".to_string(),
                version_regex: Some(r"nginx/(\d+[\.\d]*)".to_string()),
            },
            TechSignature {
                name: "IIS".to_string(),
                category: "Web Server".to_string(),
                method: DetectionMethod::Header("Server".to_string()),
                pattern: "Microsoft-IIS".to_string(),
                version_regex: Some(r"Microsoft-IIS/(\d+[\.\d]*)".to_string()),
            },
        ];
        signatures.insert("web_servers".to_string(), web_servers);

        // JavaScript frameworks
        let js_frameworks = vec![
            TechSignature {
                name: "React".to_string(),
                category: "JavaScript Framework".to_string(),
                method: DetectionMethod::Content,
                pattern: "react".to_string(),
                version_regex: Some(r"react@(\d+[\.\d]*)".to_string()),
            },
            TechSignature {
                name: "Vue.js".to_string(),
                category: "JavaScript Framework".to_string(),
                method: DetectionMethod::Content,
                pattern: "Vue".to_string(),
                version_regex: Some(r"vue@(\d+[\.\d]*)".to_string()),
            },
            TechSignature {
                name: "Angular".to_string(),
                category: "JavaScript Framework".to_string(),
                method: DetectionMethod::Content,
                pattern: "angular".to_string(),
                version_regex: Some(r"angular@(\d+[\.\d]*)".to_string()),
            },
            // Add a Script detection example
            TechSignature {
                name: "jQuery".to_string(),
                category: "JavaScript Library".to_string(),
                method: DetectionMethod::Script,
                pattern: "jquery".to_string(),
                version_regex: Some(r"jQuery v(\d+[\.\d]*)".to_string()),
            },
        ];
        signatures.insert("js_frameworks".to_string(), js_frameworks);

        // CMS platforms
        let cms_platforms = vec![
            TechSignature {
                name: "WordPress".to_string(),
                category: "CMS".to_string(),
                method: DetectionMethod::Content,
                pattern: "wp-content".to_string(),
                version_regex: None,
            },
            TechSignature {
                name: "Drupal".to_string(),
                category: "CMS".to_string(),
                method: DetectionMethod::Content,
                pattern: "Drupal".to_string(),
                version_regex: Some(r"Drupal (\d+[\.\d]*)".to_string()),
            },
            TechSignature {
                name: "Joomla".to_string(),
                category: "CMS".to_string(),
                method: DetectionMethod::Content,
                pattern: "joomla".to_string(),
                version_regex: None,
            },
        ];
        signatures.insert("cms_platforms".to_string(), cms_platforms);

        // URL pattern-based detections
        let url_patterns = vec![
            TechSignature {
                name: "WordPress".to_string(),
                category: "CMS".to_string(),
                method: DetectionMethod::Url,
                pattern: "wp-content".to_string(),
                version_regex: None,
            },
            TechSignature {
                name: "Drupal".to_string(),
                category: "CMS".to_string(),
                method: DetectionMethod::Url,
                pattern: "sites/default/files".to_string(),
                version_regex: None,
            },
        ];
        signatures.insert("url_patterns".to_string(), url_patterns);

        // Cookie-based detections
        let cookie_patterns = vec![
            TechSignature {
                name: "PHP Session".to_string(),
                category: "Web Technology".to_string(),
                method: DetectionMethod::Cookie("PHPSESSID".to_string()),
                pattern: ".".to_string(), // Match any value
                version_regex: None,
            },
            TechSignature {
                name: "ASP.NET".to_string(),
                category: "Web Technology".to_string(),
                method: DetectionMethod::Cookie("ASP.NET_SessionId".to_string()),
                pattern: ".".to_string(), // Match any value
                version_regex: None,
            },
        ];
        signatures.insert("cookie_patterns".to_string(), cookie_patterns);

        signatures
    }

    /// Extract version from content using regex pattern
    fn extract_version(&self, content: &str, regex: &str) -> Option<String> {
        regex::Regex::new(regex)
            .ok()
            .and_then(|re| re.captures(content))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Process HTTP headers to detect technologies
    async fn process_headers(&self, headers: &HeaderMap, asset_id: ID) -> Vec<TechnologyFinding> {
        let mut findings = Vec::new();

        // Process each header
        for (header_name, header_value) in headers {
            let header_str = header_name.as_str();
            let value_str = match header_value.to_str() {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Match against header-based signatures
            for signatures in self.signatures.values() {
                for signature in signatures {
                    if let DetectionMethod::Header(h) = &signature.method {
                        if h.eq_ignore_ascii_case(header_str)
                            && value_str.contains(&signature.pattern)
                        {
                            let version = signature
                                .version_regex
                                .as_ref()
                                .and_then(|re| self.extract_version(value_str, re));

                            findings.push(TechnologyFinding {
                                asset_id,
                                name: signature.name.clone(),
                                version,
                                category: Some(signature.category.clone()),
                                evidence: format!("Header {}: {}", header_str, value_str),
                            });
                        }
                    }
                }
            }
        }

        findings
    }

    /// Process HTML content to detect technologies
    fn process_content(&self, content: &str, asset_id: ID) -> Vec<TechnologyFinding> {
        let mut findings = Vec::new();

        // Match against content-based signatures
        for signatures in self.signatures.values() {
            for signature in signatures {
                match &signature.method {
                    DetectionMethod::Content => {
                        if content.contains(&signature.pattern) {
                            let version = signature
                                .version_regex
                                .as_ref()
                                .and_then(|re| self.extract_version(content, re));

                            findings.push(TechnologyFinding {
                                asset_id,
                                name: signature.name.clone(),
                                version,
                                category: Some(signature.category.clone()),
                                evidence: format!("Content match for: {}", signature.pattern),
                            });
                        }
                    }
                    DetectionMethod::Script => {
                        // Look for script tags and JavaScript patterns
                        if let Some(script_content) = self.extract_scripts(content) {
                            if script_content.contains(&signature.pattern) {
                                let version = signature
                                    .version_regex
                                    .as_ref()
                                    .and_then(|re| self.extract_version(&script_content, re));

                                findings.push(TechnologyFinding {
                                    asset_id,
                                    name: signature.name.clone(),
                                    version,
                                    category: Some(signature.category.clone()),
                                    evidence: format!("Script match for: {}", signature.pattern),
                                });
                            }
                        }
                    }
                    _ => {} // Other methods handled elsewhere
                }
            }
        }

        findings
    }

    /// Extract script content from HTML
    fn extract_scripts(&self, content: &str) -> Option<String> {
        let mut scripts = String::new();
        let re = regex::Regex::new(r"<script[^>]*>(.*?)</script>").ok()?;

        for cap in re.captures_iter(content) {
            if let Some(script) = cap.get(1) {
                scripts.push_str(script.as_str());
                scripts.push('\n');
            }
        }

        if scripts.is_empty() {
            None
        } else {
            Some(scripts)
        }
    }

    /// Process cookies to detect technologies
    fn process_cookies(&self, cookies: &str, asset_id: ID) -> Vec<TechnologyFinding> {
        let mut findings = Vec::new();

        // Match against cookie-based signatures
        for signatures in self.signatures.values() {
            for signature in signatures {
                if let DetectionMethod::Cookie(cookie_name) = &signature.method {
                    // Simple cookie parsing
                    for cookie in cookies.split(';') {
                        let parts: Vec<&str> = cookie.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let name = parts[0].trim();
                            let value = parts[1].trim();

                            if name.eq_ignore_ascii_case(cookie_name)
                                && value.contains(&signature.pattern)
                            {
                                let version = signature
                                    .version_regex
                                    .as_ref()
                                    .and_then(|re| self.extract_version(value, re));

                                findings.push(TechnologyFinding {
                                    asset_id,
                                    name: signature.name.clone(),
                                    version,
                                    category: Some(signature.category.clone()),
                                    evidence: format!("Cookie {}: {}", name, value),
                                });
                            }
                        }
                    }
                }
            }
        }

        findings
    }

    /// Process URL patterns to detect technologies
    fn process_url(&self, url: &str, asset_id: ID) -> Vec<TechnologyFinding> {
        let mut findings = Vec::new();

        // Match against URL-based signatures
        for signatures in self.signatures.values() {
            for signature in signatures {
                if let DetectionMethod::Url = signature.method {
                    if url.contains(&signature.pattern) {
                        let version = signature
                            .version_regex
                            .as_ref()
                            .and_then(|re| self.extract_version(url, re));

                        findings.push(TechnologyFinding {
                            asset_id,
                            name: signature.name.clone(),
                            version,
                            category: Some(signature.category.clone()),
                            evidence: format!("URL match for: {}", signature.pattern),
                        });
                    }
                }
            }
        }

        findings
    }
}

#[async_trait::async_trait]
impl Fingerprinter for WebFingerprinter {
    async fn fingerprint(&self, target: &str, asset_id: ID) -> DiscoveryResult {
        let mut url = target.to_string();
        if !url.starts_with("http") {
            url = format!("https://{}", url);
        }

        let mut result = DiscoveryResult::new();

        // Request the target URL
        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();

                // Store URL for URL pattern matching
                result.metadata.insert("url".to_string(), url.clone());

                // Process headers to find technologies
                let header_findings = self.process_headers(&headers, asset_id).await;
                result.technologies.extend(header_findings);

                // Check for cookies
                if let Some(cookie_header) = headers.get("set-cookie") {
                    if let Ok(cookie_str) = cookie_header.to_str() {
                        let cookie_findings = self.process_cookies(cookie_str, asset_id);
                        result.technologies.extend(cookie_findings);
                    }
                }

                // Process URL patterns
                let url_findings = self.process_url(&url, asset_id);
                result.technologies.extend(url_findings);

                // If we have a successful response, get the content and process it
                if status.is_success() {
                    if let Ok(content) = response.text().await {
                        // Store content length
                        result
                            .metadata
                            .insert("content_length".to_string(), content.len().to_string());

                        // Process content to find technologies
                        let content_findings = self.process_content(&content, asset_id);
                        result.technologies.extend(content_findings);
                    }
                }

                // Add response code and headers to metadata
                result
                    .metadata
                    .insert("status_code".to_string(), status.as_u16().to_string());
                for (name, value) in headers.iter() {
                    if let Ok(value_str) = value.to_str() {
                        result
                            .metadata
                            .insert(format!("header:{}", name), value_str.to_string());
                    }
                }
            }
            Err(err) => {
                result.metadata.insert("error".to_string(), err.to_string());
            }
        }

        result
    }
}
