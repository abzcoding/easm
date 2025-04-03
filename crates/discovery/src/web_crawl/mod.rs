use crate::results::{DiscoveredWebResource, DiscoveryResult};
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

// Basic web crawler
pub async fn crawl_url(target_url: &str, depth: u8) -> Result<DiscoveryResult> {
    tracing::debug!("Crawling URL: {} with depth: {}", target_url, depth);
    let client = Client::builder()
        .user_agent("EASM Discovery Bot/0.1") // Be a good bot citizen
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let mut discovery_result = DiscoveryResult::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue: std::collections::VecDeque<(String, u8)> = std::collections::VecDeque::new();

    let base_url = Url::parse(target_url)?;
    queue.push_back((target_url.to_string(), 0));
    visited.insert(target_url.to_string());

    let source = format!("web_crawl_for_{}", base_url.host_str().unwrap_or("unknown"));

    while let Some((current_url, current_depth)) = queue.pop_front() {
        if current_depth > depth {
            continue;
        }

        tracing::trace!("Fetching: {}", current_url);
        match client.get(&current_url).send().await {
            Ok(response) => {
                let status = response.status();
                // Store headers for technology detection
                let _headers = response.headers().clone();
                let final_url = response.url().to_string(); // URL after potential redirects

                match response.text().await {
                    Ok(body) => {
                        let document = Html::parse_document(&body);
                        let title_selector = Selector::parse("title").unwrap();
                        let title = document
                            .select(&title_selector)
                            .next()
                            .map(|t| t.text().collect::<String>().trim().to_string());

                        // Implement basic technology detection
                        let mut technologies = Vec::new();

                        // Check for common technology indicators in the HTML
                        detect_technologies(&document, &mut technologies);

                        discovery_result.web_resources.push(DiscoveredWebResource {
                            url: final_url.clone(),
                            status_code: status.as_u16(),
                            title,
                            technologies,
                            source: source.clone(),
                        });

                        // Find links if depth allows further crawling
                        if current_depth < depth {
                            let link_selector = Selector::parse("a[href]").unwrap();
                            for element in document.select(&link_selector) {
                                if let Some(href) = element.value().attr("href") {
                                    match base_url.join(href) {
                                        Ok(mut next_url) => {
                                            next_url.set_fragment(None); // Ignore fragments
                                            let next_url_str = next_url.to_string();

                                            // Stay on the same domain (optional, configurable?)
                                            if next_url.host_str() == base_url.host_str()
                                                && !visited.contains(&next_url_str)
                                            {
                                                visited.insert(next_url_str.clone());
                                                queue.push_back((next_url_str, current_depth + 1));
                                            }
                                        }
                                        Err(e) => {
                                            tracing::trace!(
                                                "Failed to parse relative URL {}: {}",
                                                href,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read body for {}: {}", final_url, e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch {}: {}", current_url, e);
            }
        }
    }

    Ok(discovery_result)
}

// Helper function to detect web technologies from HTML
fn detect_technologies(document: &Html, technologies: &mut Vec<String>) {
    // Check for common JS frameworks
    if let Ok(selector) = Selector::parse("script[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src") {
                if src.contains("react") || src.contains("react.") {
                    technologies.push("React".to_string());
                } else if src.contains("angular") {
                    technologies.push("Angular".to_string());
                } else if src.contains("vue") {
                    technologies.push("Vue.js".to_string());
                } else if src.contains("jquery") {
                    technologies.push("jQuery".to_string());
                }
            }
        }
    }

    // Check for meta tags
    if let Ok(selector) = Selector::parse("meta[name='generator']") {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                if content.contains("WordPress") {
                    technologies.push("WordPress".to_string());
                } else if content.contains("Drupal") {
                    technologies.push("Drupal".to_string());
                } else if content.contains("Joomla") {
                    technologies.push("Joomla".to_string());
                } else if !content.is_empty() {
                    technologies.push(format!("Generator: {}", content));
                }
            }
        }
    }

    // Check for common CSS frameworks
    if let Ok(selector) = Selector::parse("link[rel='stylesheet']") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if href.contains("bootstrap") {
                    technologies.push("Bootstrap".to_string());
                } else if href.contains("tailwind") {
                    technologies.push("Tailwind CSS".to_string());
                } else if href.contains("foundation") {
                    technologies.push("Foundation".to_string());
                }
            }
        }
    }
}
