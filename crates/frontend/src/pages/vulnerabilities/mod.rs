use crate::components::ui::vulnerability_card::{Vulnerability, VulnerabilityCard};
use leptos::prelude::*;

#[component]
pub fn VulnerabilitiesPage() -> impl IntoView {
    // Mock data - in a real app, this would be fetched from API
    let vulnerabilities = vec![
        Vulnerability {
            id: "1".to_string(),
            title: "Outdated WordPress Version".to_string(),
            description: "The WordPress installation is using an outdated version (5.8.1) with known security vulnerabilities.".to_string(),
            severity: "High".to_string(),
            status: "Open".to_string(),
            asset_name: "example.com".to_string(),
            discovery_date: "2023-03-15".to_string(),
        },
        Vulnerability {
            id: "2".to_string(),
            title: "SSL Certificate Expiring Soon".to_string(),
            description: "The SSL certificate for this domain will expire in 14 days, which may lead to browser warnings.".to_string(),
            severity: "Medium".to_string(),
            status: "In Progress".to_string(),
            asset_name: "api.example.com".to_string(),
            discovery_date: "2023-03-16".to_string(),
        },
        Vulnerability {
            id: "3".to_string(),
            title: "Open SSH Port".to_string(),
            description: "SSH port 22 is open and accessible from the internet, consider restricting access.".to_string(),
            severity: "Low".to_string(),
            status: "Open".to_string(),
            asset_name: "192.168.1.1".to_string(),
            discovery_date: "2023-03-14".to_string(),
        },
        Vulnerability {
            id: "4".to_string(),
            title: "Cross-Site Scripting (XSS)".to_string(),
            description: "A cross-site scripting vulnerability was detected in the search functionality.".to_string(),
            severity: "Critical".to_string(),
            status: "Open".to_string(),
            asset_name: "example.org".to_string(),
            discovery_date: "2023-03-12".to_string(),
        },
        Vulnerability {
            id: "5".to_string(),
            title: "Insecure HTTP Access".to_string(),
            description: "The site allows non-HTTPS access, which can lead to data interception.".to_string(),
            severity: "Medium".to_string(),
            status: "Fixed".to_string(),
            asset_name: "blog.example.com".to_string(),
            discovery_date: "2023-03-10".to_string(),
        },
    ];

    let (vulnerabilities_signal, _set_vulnerabilities) = signal(vulnerabilities);

    // We don't use this directly - marking as unused with _
    let _handle_vuln_click = move |id: String| {
        log::info!("Vulnerability selected: {}", id);
        // In a real app, this would navigate to vulnerability details or open a modal
    };

    let total_count = move || vulnerabilities_signal.get().len();
    let open_count = move || {
        vulnerabilities_signal
            .get()
            .iter()
            .filter(|v| v.status == "Open")
            .count()
    };
    let critical_count = move || {
        vulnerabilities_signal
            .get()
            .iter()
            .filter(|v| v.severity == "Critical")
            .count()
    };
    let high_count = move || {
        vulnerabilities_signal
            .get()
            .iter()
            .filter(|v| v.severity == "High")
            .count()
    };

    view! {
        <div class="page-container">
            <div class="page-header">
                <h1 class="page-title">"Vulnerabilities"</h1>
                <div class="page-actions">
                    <button class="btn btn-primary">"Export Report"</button>
                </div>
            </div>

            <div class="grid grid-3">
                <div class="card">
                    <h2>"Overview"</h2>
                    <div class="stat">
                        <span class="stat-value">{move || total_count().to_string()}</span>
                        <span class="stat-label">"Total Vulnerabilities"</span>
                    </div>
                </div>

                <div class="card">
                    <h2>"Open Issues"</h2>
                    <div class="stat">
                        <span class="stat-value">{move || open_count().to_string()}</span>
                        <span class="stat-label">"Open Vulnerabilities"</span>
                    </div>
                </div>

                <div class="card">
                    <h2>"Critical & High"</h2>
                    <div class="stat">
                        <span class="stat-value">{move || (critical_count() + high_count()).to_string()}</span>
                        <span class="stat-label">"High Priority Issues"</span>
                    </div>
                </div>
            </div>

            <div class="filter-bar">
                <input type="text" placeholder="Search vulnerabilities..." class="search-input" />
                <select class="filter-select">
                    <option>"All Severities"</option>
                    <option>"Critical"</option>
                    <option>"High"</option>
                    <option>"Medium"</option>
                    <option>"Low"</option>
                </select>
                <select class="filter-select">
                    <option>"All Statuses"</option>
                    <option>"Open"</option>
                    <option>"In Progress"</option>
                    <option>"Fixed"</option>
                </select>
            </div>

            <div class="vulnerability-grid">
                {move || vulnerabilities_signal.get().iter().map(|vuln| {
                    let vuln_id = vuln.id.clone();
                    let on_click_callback = Callback::new(move |_| {
                        log::info!("Vulnerability selected: {}", vuln_id);
                    });

                    view! {
                        <VulnerabilityCard
                            vulnerability={vuln.clone()}
                            on_click={on_click_callback}
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
