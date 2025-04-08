#[cfg(test)]
mod end_to_end_workflow_tests {
    use backend::models::{Asset, DiscoveryJob, Organization, Port, Technology, Vulnerability};
    use backend::services::{AssetServiceImpl, DiscoveryServiceImpl, VulnerabilityServiceImpl};
    use backend::traits::{AssetService, VulnerabilityService};
    use chrono::Utc;
    use discovery::results::{DiscoveredDomain, DiscoveredIp, DiscoveryResult};
    use infrastructure::repositories::RepositoryFactory;
    use serde_json::json;
    use shared::{
        config::Config,
        types::{AssetType, JobStatus, JobType, PortStatus, Protocol, Severity},
    };
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::str::FromStr;
    use uuid::Uuid;

    // Mock discovery task that simulates finding subdomains, IPs, and ports
    struct MockDiscoveryTask;

    impl MockDiscoveryTask {
        async fn execute_dns_enumeration(
            &self,
            target_domain: &str,
        ) -> anyhow::Result<DiscoveryResult> {
            let mut result = DiscoveryResult::new();

            // Add synthetic subdomains
            for name in ["www", "api", "admin", "mail", "dev"].iter() {
                result.domains.push(DiscoveredDomain {
                    domain_name: format!("{}.{}", name, target_domain),
                    source: "mock_dns_enumeration".to_string(),
                });
            }

            Ok(result)
        }

        async fn execute_ip_resolution(
            &self,
            domains: &[String],
        ) -> anyhow::Result<DiscoveryResult> {
            let mut result = DiscoveryResult::new();
            let mut ip_counter = 1;

            for domain in domains {
                // Generate a synthetic IP address for each domain
                let ip_str = format!("192.168.1.{}", ip_counter);
                let ip_addr = IpAddr::from_str(&ip_str).unwrap();

                result.ip_addresses.push(DiscoveredIp {
                    ip_address: ip_addr,
                    source: "mock_ip_resolution".to_string(),
                });

                ip_counter += 1;
            }

            Ok(result)
        }

        async fn execute_port_scan(&self, ips: &[String]) -> anyhow::Result<DiscoveryResult> {
            let mut result = DiscoveryResult::new();

            // Common ports to check
            let common_ports = [
                (80, Protocol::TCP, "http", "Apache/2.4.41"),
                (443, Protocol::TCP, "https", "nginx/1.18.0"),
                (22, Protocol::TCP, "ssh", "OpenSSH_8.2p1"),
                (25, Protocol::TCP, "smtp", "Postfix"),
                (53, Protocol::UDP, "domain", ""),
            ];

            for ip in ips {
                let ip_addr = IpAddr::from_str(ip).unwrap();
                // Add a subset of ports to each IP
                for (i, (port_number, protocol, service, banner)) in
                    common_ports.iter().enumerate().take(3)
                {
                    // Skip some ports randomly to simulate variety
                    if (i as u8 + ip.as_bytes()[ip.len() - 1]) % 3 != 0 {
                        result.ports.push(discovery::port_scan::DiscoveredPort {
                            ip_address: ip_addr,
                            port: *port_number,
                            protocol: format!("{:?}", protocol),
                            status: "OPEN".to_string(),
                            service_name: Some(service.to_string()),
                            banner: if banner.is_empty() {
                                None
                            } else {
                                Some(banner.to_string())
                            },
                            source: "mock_port_scan".to_string(),
                        });
                    }
                }
            }

            Ok(result)
        }

        async fn execute_web_discovery(
            &self,
            domains: &[String],
        ) -> anyhow::Result<DiscoveryResult> {
            let mut result = DiscoveryResult::new();

            for domain in domains {
                // Only create web app entries for domains that would typically have them
                if domain.starts_with("www.")
                    || domain.starts_with("api.")
                    || domain.starts_with("dev.")
                {
                    let web_url = format!("https://{}", domain);

                    // Add tech stack info to the metadata
                    let tech_stack = if domain.starts_with("www.") {
                        json!({
                            "server": "nginx/1.18.0",
                            "technologies": ["WordPress 5.9", "PHP 7.4.3", "jQuery 3.5.1"]
                        })
                    } else if domain.starts_with("api.") {
                        json!({
                            "server": "Apache/2.4.41",
                            "technologies": ["Node.js 14.17.0", "Express 4.17.1"]
                        })
                    } else {
                        json!({
                            "server": "nginx/1.18.0",
                            "technologies": ["React 17.0.2", "Bootstrap 5.1.3"]
                        })
                    };

                    result
                        .web_resources
                        .push(discovery::results::DiscoveredWebResource {
                            url: web_url.clone(),
                            status_code: 200,
                            title: Some(format!("{} - Homepage", domain)),
                            technologies: vec![], // These will be added to metadata
                            source: "mock_web_discovery".to_string(),
                        });

                    result.metadata.insert(web_url, tech_stack.to_string());
                }
            }

            Ok(result)
        }

        async fn execute_vulnerability_scan(
            &self,
            assets: &[Asset],
            ports: &[Port],
        ) -> anyhow::Result<Vec<Vulnerability>> {
            let mut vulnerabilities = Vec::new();

            // Map of IP address assets by their value
            let ip_assets: HashMap<_, _> = assets
                .iter()
                .filter(|a| a.asset_type == AssetType::IPAddress)
                .map(|a| (a.value.clone(), a))
                .collect();

            // Map of ports by IP address
            let mut ip_ports: HashMap<String, Vec<&Port>> = HashMap::new();
            for port in ports {
                if let Some(ip_asset) = assets.iter().find(|a| a.id == port.asset_id) {
                    ip_ports
                        .entry(ip_asset.value.clone())
                        .or_default()
                        .push(port);
                }
            }

            // Map of web apps by domain
            let web_apps: HashMap<_, _> = assets
                .iter()
                .filter(|a| a.asset_type == AssetType::WebApp)
                .filter_map(|a| {
                    let attrs = &a.attributes;
                    attrs
                        .get("domain")
                        .and_then(|d| d.as_str())
                        .map(|domain| (domain.to_string(), a))
                })
                .collect();

            // 1. Check for SSH vulnerabilities
            for (ip, ports) in ip_ports.iter() {
                if let Some(ssh_port) = ports.iter().find(|p| p.port_number == 22) {
                    if let Some(ip_asset) = ip_assets.get(ip) {
                        // Create SSH weak crypto vulnerability if version < 8.0
                        if let Some(banner) = &ssh_port.banner {
                            if banner.contains("7.") || banner.contains("6.") {
                                let vuln = Vulnerability::new(
                                    ip_asset.id,
                                    Some(ssh_port.id),
                                    "OpenSSH Weak Cryptographic Algorithms".to_string(),
                                    Some(
                                        "SSH server allows weak cryptographic algorithms"
                                            .to_string(),
                                    ),
                                    Severity::Medium,
                                    None,
                                    Some(json!({
                                        "affected_algorithms": ["diffie-hellman-group1-sha1", "ssh-dss"],
                                        "banner": banner
                                    })),
                                    Some(
                                        "Update SSH configuration to disable weak algorithms"
                                            .to_string(),
                                    ),
                                );
                                vulnerabilities.push(vuln);
                            }
                        }
                    }
                }
            }

            // 2. Check for web vulnerabilities
            for asset in assets.iter().filter(|a| a.asset_type == AssetType::WebApp) {
                println!("Checking web app for vulnerabilities: {}", asset.value);
                // WordPress vulnerabilities
                let attrs = &asset.attributes;

                if let Some(techs) = attrs.get("technologies") {
                    println!("Found technologies attribute: {:?}", techs);
                    let techs_array = techs.as_array().cloned().unwrap_or_default();

                    for tech in techs_array {
                        if let Some(tech_name) = tech.as_str() {
                            println!("Checking technology: {}", tech_name);

                            if tech_name.contains("WordPress") {
                                println!("Found WordPress! Creating high severity vulnerability");
                                // Add XSS vulnerability
                                let vuln1 = Vulnerability::new(
                                    asset.id,
                                    None,
                                    "WordPress XSS Vulnerability".to_string(),
                                    Some("Cross-site scripting vulnerability in search functionality".to_string()),
                                    Severity::High,
                                    Some("CVE-2022-1234".to_string()),
                                    Some(json!({
                                        "url": format!("{}/search?q=<script>alert(1)</script>", asset.value),
                                        "parameter": "q"
                                    })),
                                    Some("Update WordPress to the latest version".to_string()),
                                );
                                vulnerabilities.push(vuln1);

                                // Add outdated plugin vulnerability
                                let vuln2 = Vulnerability::new(
                                    asset.id,
                                    None,
                                    "WordPress Outdated Plugin".to_string(),
                                    Some("Contact Form 7 plugin is outdated and contains security vulnerabilities".to_string()),
                                    Severity::Medium,
                                    Some("CVE-2022-5678".to_string()),
                                    Some(json!({
                                        "plugin": "Contact Form 7",
                                        "current_version": "5.4.1",
                                        "latest_version": "5.5.2"
                                    })),
                                    Some("Update the Contact Form 7 plugin to the latest version".to_string()),
                                );
                                vulnerabilities.push(vuln2);
                            }

                            // Node.js vulnerabilities
                            if tech_name.contains("Node.js") {
                                println!("Found Node.js! Creating high severity vulnerability");
                                // Add prototype pollution vulnerability
                                let vuln = Vulnerability::new(
                                    asset.id,
                                    None,
                                    "Node.js Prototype Pollution".to_string(),
                                    Some(
                                        "Prototype pollution vulnerability in lodash dependency"
                                            .to_string(),
                                    ),
                                    Severity::High,
                                    Some("CVE-2021-23337".to_string()),
                                    Some(json!({
                                        "package": "lodash",
                                        "current_version": "4.17.15",
                                        "fixed_version": "4.17.21"
                                    })),
                                    Some("Update lodash to version 4.17.21 or later".to_string()),
                                );
                                vulnerabilities.push(vuln);
                            }
                        }
                    }

                    // TLS certificate vulnerabilities
                    if asset.value.starts_with("https://") {
                        // Random TLS issue based on hostname hash
                        let hostname_hash = asset
                            .value
                            .as_bytes()
                            .iter()
                            .map(|b| *b as u32)
                            .sum::<u32>();
                        if hostname_hash % 5 == 0 {
                            let vuln = Vulnerability::new(
                                asset.id,
                                None,
                                "TLS Certificate Issues".to_string(),
                                Some("TLS certificate is self-signed or uses weak signature algorithm".to_string()),
                                Severity::Medium,
                                None,
                                Some(json!({
                                    "issue": "self-signed",
                                    "signature_algorithm": "SHA1withRSA"
                                })),
                                Some("Replace with a trusted TLS certificate using modern signature algorithms".to_string()),
                            );
                            vulnerabilities.push(vuln);
                        }
                    }
                } else {
                    println!(
                        "No technologies attribute found for web app: {}",
                        asset.value
                    );
                }
            }

            println!(
                "Created {} vulnerabilities, with {} high severity",
                vulnerabilities.len(),
                vulnerabilities
                    .iter()
                    .filter(|v| v.severity == Severity::High)
                    .count()
            );

            Ok(vulnerabilities)
        }
    }

    async fn setup_test_db() -> RepositoryFactory {
        let _ = dotenvy::dotenv();
        let config = Config::from_env().expect("Failed to load config");
        let database_url = config.database_url.clone();

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        RepositoryFactory::new(pool)
    }

    async fn create_test_organization(factory: &RepositoryFactory) -> Organization {
        let org_repo = factory.organization_repository();

        // Create a test organization
        let org_name = format!("EASM E2E Test Org {}", Uuid::new_v4());
        let org = Organization::new(org_name);
        let org = org_repo
            .create_organization(&org)
            .await
            .expect("Failed to create organization");

        org
    }

    #[tokio::test]
    async fn test_complete_easm_workflow() {
        // Set up database and repositories
        let factory = setup_test_db().await;
        let org = create_test_organization(&factory).await;

        // Services
        let asset_service = AssetServiceImpl::new(factory.asset_repository());
        let discovery_service = DiscoveryServiceImpl::new(
            factory.asset_repository(),
            factory.discovery_job_repository(),
        );
        let vulnerability_service = VulnerabilityServiceImpl::new(
            factory.vulnerability_repository(),
            factory.asset_repository(),
        );

        // Discovery task
        let discovery_task = MockDiscoveryTask;

        // 1. Start with a target domain
        let root_domain = format!("easm-{}.example.com", Uuid::new_v4().as_simple());
        println!("Starting EASM workflow with domain: {}", root_domain);

        // Create initial domain asset
        let domain_asset = Asset::new(org.id, AssetType::Domain, root_domain.clone(), None);

        let created_domain = asset_service
            .create_asset(&domain_asset)
            .await
            .expect("Failed to create initial domain asset");

        // 2. Create and run DNS enumeration discovery job
        let dns_job = DiscoveryJob::new(org.id, JobType::DnsEnum, Some(root_domain.clone()), None);

        let dns_job = discovery_service
            .create_job(org.id, JobType::DnsEnum, Some(root_domain.clone()), None)
            .await
            .expect("Failed to create DNS enumeration job");

        // Execute DNS enumeration
        let dns_results = discovery_task
            .execute_dns_enumeration(&root_domain)
            .await
            .expect("Failed to execute DNS enumeration");

        println!("Discovered {} subdomains", dns_results.domains.len());

        // Save subdomains as assets
        let mut all_domains = vec![root_domain.clone()];
        let mut domain_assets = vec![created_domain];

        for domain in dns_results.domains.iter() {
            let asset = Asset::new(
                org.id,
                AssetType::Domain,
                domain.domain_name.clone(),
                Some(json!({
                    "source": domain.source,
                    "parent_domain": root_domain,
                })),
            );

            let created = asset_service
                .create_asset(&asset)
                .await
                .expect("Failed to create subdomain asset");

            all_domains.push(domain.domain_name.clone());
            domain_assets.push(created);
        }

        // Update job status
        let mut completed_dns_job = dns_job.clone();
        completed_dns_job.status = JobStatus::Completed;
        completed_dns_job.completed_at = Some(Utc::now());

        let _ = discovery_service
            .update_job(&completed_dns_job)
            .await
            .expect("Failed to update DNS job status");

        // 3. Run IP resolution for all domains
        let ip_job = DiscoveryJob::new(
            org.id,
            JobType::PortScan,
            None,
            Some(json!({
                "domains": all_domains
            })),
        );

        let ip_job = discovery_service
            .create_job(
                org.id,
                JobType::PortScan,
                None,
                Some(json!({
                    "domains": all_domains
                })),
            )
            .await
            .expect("Failed to create IP resolution job");

        // Execute IP resolution
        let ip_results = discovery_task
            .execute_ip_resolution(&all_domains)
            .await
            .expect("Failed to execute IP resolution");

        println!("Discovered {} IP addresses", ip_results.ip_addresses.len());

        // Save IP addresses as assets
        let mut ip_assets = Vec::new();
        let mut all_ips = Vec::new();

        for ip in ip_results.ip_addresses.iter() {
            let ip_string = ip.ip_address.to_string();
            let asset = Asset::new(
                org.id,
                AssetType::IPAddress,
                ip_string.clone(),
                Some(json!({
                    "source": ip.source,
                })),
            );

            let created = asset_service
                .create_asset(&asset)
                .await
                .expect("Failed to create IP asset");

            all_ips.push(ip_string);
            ip_assets.push(created);
        }

        // Update job status
        let mut completed_ip_job = ip_job.clone();
        completed_ip_job.status = JobStatus::Completed;
        completed_ip_job.completed_at = Some(Utc::now());

        let _ = discovery_service
            .update_job(&completed_ip_job)
            .await
            .expect("Failed to update IP job status");

        // 4. Run port scanning on discovered IPs
        let port_job = DiscoveryJob::new(
            org.id,
            JobType::PortScan,
            None,
            Some(json!({
                "ips": all_ips
            })),
        );

        let port_job = discovery_service
            .create_job(
                org.id,
                JobType::PortScan,
                None,
                Some(json!({
                    "ips": all_ips
                })),
            )
            .await
            .expect("Failed to create port scan job");

        // Execute port scanning
        let port_results = discovery_task
            .execute_port_scan(&all_ips)
            .await
            .expect("Failed to execute port scan");

        println!("Discovered {} open ports", port_results.ports.len());

        // Save ports
        let mut created_ports = Vec::new();

        for discovered_port in port_results.ports.iter() {
            // Find the asset ID for this IP
            let ip_string = discovered_port.ip_address.to_string();
            if let Some(ip_asset) = ip_assets.iter().find(|a| a.value == ip_string) {
                let port = Port {
                    id: Uuid::new_v4(),
                    asset_id: ip_asset.id,
                    port_number: discovered_port.port as i32,
                    protocol: match discovered_port.protocol.as_str() {
                        "TCP" => Protocol::TCP,
                        "UDP" => Protocol::UDP,
                        _ => Protocol::TCP,
                    },
                    service_name: discovered_port.service_name.clone(),
                    banner: discovered_port.banner.clone(),
                    status: PortStatus::Open,
                    first_seen: Utc::now(),
                    last_seen: Utc::now(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                let created = factory
                    .port_repository()
                    .create_port(&port)
                    .await
                    .expect("Failed to create port");

                created_ports.push(created);
            }
        }

        // Update job status
        let mut completed_port_job = port_job.clone();
        completed_port_job.status = JobStatus::Completed;
        completed_port_job.completed_at = Some(Utc::now());

        let _ = discovery_service
            .update_job(&completed_port_job)
            .await
            .expect("Failed to update port scan job status");

        // 5. Web application discovery
        let web_job = DiscoveryJob::new(
            org.id,
            JobType::WebCrawl,
            None,
            Some(json!({
                "domains": all_domains
            })),
        );

        let web_job = discovery_service
            .create_job(
                org.id,
                JobType::WebCrawl,
                None,
                Some(json!({
                    "domains": all_domains
                })),
            )
            .await
            .expect("Failed to create web discovery job");

        // Execute web discovery
        let web_results = discovery_task
            .execute_web_discovery(&all_domains)
            .await
            .expect("Failed to execute web discovery");

        println!(
            "Discovered {} web applications",
            web_results.web_resources.len()
        );

        // Save web applications as assets
        let mut web_assets = Vec::new();

        for web_resource in web_results.web_resources.iter() {
            // Extract domain from URL
            let domain = web_resource
                .url
                .replace("https://", "")
                .replace("http://", "");

            let tech_stack_str = web_results
                .metadata
                .get(&web_resource.url)
                .cloned()
                .unwrap_or_default();
            println!("Technology stack string: {}", tech_stack_str);
            let tech_stack: serde_json::Value =
                serde_json::from_str(&tech_stack_str).unwrap_or(json!({}));
            println!("Parsed tech stack: {}", tech_stack);

            let mut attributes = json!({
                "domain": domain,
                "status_code": web_resource.status_code,
                "title": web_resource.title,
                "technologies": tech_stack.get("technologies").cloned().unwrap_or(json!([]))
            });

            // Merge with discovered attributes
            if let Some(obj) = attributes.as_object_mut() {
                if let Some(tech_obj) = tech_stack.as_object() {
                    for (k, v) in tech_obj {
                        if k != "technologies" {
                            // Skip technologies as we've already added them
                            println!("Adding attribute: {} = {}", k, v);
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            println!("Final attributes: {}", attributes);

            let asset = Asset::new(
                org.id,
                AssetType::WebApp,
                web_resource.url.clone(),
                Some(attributes.clone()), // Clone here so we can still access attributes
            );

            let created = asset_service
                .create_asset(&asset)
                .await
                .expect("Failed to create web app asset");

            web_assets.push(created.clone());

            // Create technology records from attributes
            if let Some(tech_list) = attributes.get("technologies").and_then(|t| t.as_array()) {
                println!("Found technologies array: {:?}", tech_list);
                for tech_item in tech_list {
                    if let Some(tech_str) = tech_item.as_str() {
                        println!("Processing technology: {}", tech_str);
                        let parts: Vec<&str> = tech_str.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let tech_name = parts[0].to_string();
                            let tech_version = parts[1].to_string();
                            println!("Creating technology: {} {}", tech_name, tech_version);

                            let tech = Technology {
                                id: Uuid::new_v4(),
                                asset_id: created.id,
                                name: tech_name.clone(),
                                version: Some(tech_version),
                                category: Some(
                                    match tech_name.as_str() {
                                        "WordPress" => "CMS",
                                        "PHP" => "Programming Language",
                                        "jQuery" => "JavaScript Library",
                                        "Node.js" => "Runtime Environment",
                                        "Express" => "Web Framework",
                                        "React" => "JavaScript Framework",
                                        "Bootstrap" => "CSS Framework",
                                        _ => "Other",
                                    }
                                    .to_string(),
                                ),
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                            };

                            let _ = factory
                                .technology_repository()
                                .create_technology(&tech)
                                .await
                                .expect("Failed to create technology");
                        }
                    }
                }
            } else {
                println!("No technologies array found in attributes");
            }

            // Create server technology if present
            if let Some(server) = attributes.get("server").and_then(|s| s.as_str()) {
                println!("Found server: {}", server);
                let parts: Vec<&str> = server.split('/').collect();
                if parts.len() >= 2 {
                    let tech_name = parts[0].to_string();
                    let tech_version = parts[1].to_string();
                    println!("Creating server technology: {} {}", tech_name, tech_version);

                    let tech = Technology {
                        id: Uuid::new_v4(),
                        asset_id: created.id,
                        name: tech_name,
                        version: Some(tech_version),
                        category: Some("Web Server".to_string()),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    let _ = factory
                        .technology_repository()
                        .create_technology(&tech)
                        .await
                        .expect("Failed to create web server technology");
                }
            } else {
                println!("No server technology found in attributes");
            }
        }

        // Update job status
        let mut completed_web_job = web_job.clone();
        completed_web_job.status = JobStatus::Completed;
        completed_web_job.completed_at = Some(Utc::now());

        let _ = discovery_service
            .update_job(&completed_web_job)
            .await
            .expect("Failed to update web discovery job status");

        // Combine all assets for vulnerability scanning
        let all_assets: Vec<Asset> = domain_assets
            .into_iter()
            .chain(ip_assets.into_iter())
            .chain(web_assets.into_iter())
            .collect();

        // 6. Vulnerability scanning
        let vuln_job = DiscoveryJob::new(
            org.id,
            JobType::VulnScan,
            None,
            Some(json!({
                "asset_count": all_assets.len()
            })),
        );

        let vuln_job = discovery_service
            .create_job(
                org.id,
                JobType::VulnScan,
                None,
                Some(json!({
                    "asset_count": all_assets.len()
                })),
            )
            .await
            .expect("Failed to create vulnerability scan job");

        // Execute vulnerability scanning
        let vulnerabilities = discovery_task
            .execute_vulnerability_scan(&all_assets, &created_ports)
            .await
            .expect("Failed to execute vulnerability scan");

        println!("Discovered {} vulnerabilities", vulnerabilities.len());

        // Save vulnerabilities
        let mut created_vulns = Vec::new();

        for vuln in vulnerabilities.iter() {
            let created = vulnerability_service
                .create_vulnerability(vuln)
                .await
                .expect("Failed to create vulnerability");

            created_vulns.push(created);
        }

        // Update job status
        let mut completed_vuln_job = vuln_job.clone();
        completed_vuln_job.status = JobStatus::Completed;
        completed_vuln_job.completed_at = Some(Utc::now());

        let _ = discovery_service
            .update_job(&completed_vuln_job)
            .await
            .expect("Failed to update vulnerability scan job status");

        // 7. Validate data

        // Verify assets exist
        let domain_count = asset_service
            .count_assets(Some(org.id), Some(AssetType::Domain), None)
            .await
            .expect("Failed to count domain assets");

        let ip_count = asset_service
            .count_assets(Some(org.id), Some(AssetType::IPAddress), None)
            .await
            .expect("Failed to count IP assets");

        let web_count = asset_service
            .count_assets(Some(org.id), Some(AssetType::WebApp), None)
            .await
            .expect("Failed to count web app assets");

        println!(
            "Asset counts - Domains: {}, IPs: {}, Web Apps: {}",
            domain_count, ip_count, web_count
        );

        assert!(domain_count >= 5); // Root domain + at least 4 subdomains
        assert!(ip_count >= 5); // At least one IP per domain
        assert!(web_count >= 3); // At least 3 web applications

        // Verify technology count
        let techs = factory
            .technology_repository()
            .list_technologies(Some(org.id), None, None, 100, 0)
            .await
            .expect("Failed to get technologies");

        println!("Discovered {} technologies", techs.len());
        assert!(techs.len() >= 5); // At least 5 technologies

        // Verify vulnerability stats
        let vuln_count = vulnerability_service
            .count_vulnerabilities(Some(org.id), None, None, None)
            .await
            .expect("Failed to count vulnerabilities");

        let high_severity_vulns = vulnerability_service
            .list_vulnerabilities(Some(org.id), None, Some(Severity::High), None, 100, 0)
            .await
            .expect("Failed to list high severity vulnerabilities");

        println!(
            "Vulnerabilities - Total: {}, High: {}",
            vuln_count,
            high_severity_vulns.len()
        );

        assert!(vuln_count >= 3); // At least 3 vulnerabilities
        assert!(!high_severity_vulns.is_empty()); // At least 1 high severity

        // Clean up created resources
        println!("Cleaning up test data...");

        // Delete vulnerabilities
        for vuln in created_vulns {
            let _ = vulnerability_service
                .delete_vulnerability(vuln.id)
                .await
                .expect("Failed to delete vulnerability");
        }

        // Delete jobs
        let jobs = [dns_job.id, ip_job.id, port_job.id, web_job.id, vuln_job.id];
        for job_id in jobs {
            let _ = factory
                .discovery_job_repository()
                .delete_job(job_id)
                .await
                .expect("Failed to delete job");
        }

        // Delete organization (will cascade delete assets)
        let _ = factory
            .organization_repository()
            .delete_organization(org.id)
            .await
            .expect("Failed to delete organization");

        println!("EASM workflow test completed successfully");
    }
}
