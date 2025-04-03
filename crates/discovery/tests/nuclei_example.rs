#![cfg(test)]

use discovery::tasks::{DiscoveryTask, DiscoveryTaskType, NucleiTaskParams};
use discovery::vulnerability::nuclei::NucleiRunner;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Ignore by default as it requires the nuclei binary to be installed
async fn test_nuclei_direct() {
    // Direct use of NucleiRunner
    let scanner = NucleiRunner::new()
        .with_severity("medium,high,critical".to_string())
        .with_rate_limit(50);

    let targets = vec!["https://example.com".to_string()]; // Replace with actual target

    match scanner.scan_targets(&targets).await {
        Ok(results) => {
            println!(
                "Discovered {} vulnerabilities",
                results.raw_vulnerabilities.len()
            );
            for vuln in &results.raw_vulnerabilities {
                println!(
                    "[{}] {} - {} ({})",
                    vuln.severity, vuln.name, vuln.target, vuln.template_id
                );
            }
        }
        Err(e) => {
            eprintln!("Error scanning with Nuclei: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Ignore by default as it requires the nuclei binary to be installed
async fn test_nuclei_via_task() {
    // Use Nuclei through the task system
    let task = DiscoveryTask {
        job_id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        task_type: DiscoveryTaskType::VulnerabilityScanNuclei,
        target: "https://example.com".to_string(), // Replace with actual target
        nuclei_params: Some(NucleiTaskParams {
            severity: Some("medium,high,critical".to_string()),
            rate_limit: Some(50),
            templates: Some(vec!["cves".to_string()]), // Only use CVE templates
            ..Default::default()
        }),
    };

    match task.execute().await {
        Ok(results) => {
            println!(
                "Discovered {} vulnerabilities",
                results.raw_vulnerabilities.len()
            );
            for vuln in &results.raw_vulnerabilities {
                println!(
                    "[{}] {} - {} ({})",
                    vuln.severity, vuln.name, vuln.target, vuln.template_id
                );

                if let Some(cve) = &vuln.cve_id {
                    println!("  CVE: {}", cve);
                }

                if let Some(score) = vuln.cvss_score {
                    println!("  CVSS: {}", score);
                }

                if !vuln.references.is_empty() {
                    println!("  References:");
                    for reference in &vuln.references {
                        println!("    - {}", reference);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error executing Nuclei task: {}", e);
        }
    }
}
