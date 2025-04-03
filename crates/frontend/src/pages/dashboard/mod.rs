use crate::components::ui::chart::{Chart, ChartData, ChartDataset, ChartType};
use crate::utils::{format_date, format_severity, severity_class, truncate};
use leptos::prelude::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    // Create chart data
    let vulnerability_chart_data = ChartData {
        labels: vec![
            "Critical".to_string(),
            "High".to_string(),
            "Medium".to_string(),
            "Low".to_string(),
            "Info".to_string(),
        ],
        datasets: vec![ChartDataset {
            label: "Vulnerabilities".to_string(),
            data: vec![5.0, 15.0, 12.0, 8.0, 2.0],
            background_colors: Some(vec![
                "#7E1F1FFF".to_string(),
                "#D63939FF".to_string(),
                "#F59F00FF".to_string(),
                "#2FB344FF".to_string(),
                "#4299E1FF".to_string(),
            ]),
            border_colors: Some(vec![
                "#7E1F1F".to_string(),
                "#D63939".to_string(),
                "#F59F00".to_string(),
                "#2FB344".to_string(),
                "#4299E1".to_string(),
            ]),
        }],
    };

    let asset_chart_data = ChartData {
        labels: vec![
            "Domains".to_string(),
            "IPs".to_string(),
            "Subdomains".to_string(),
            "URLs".to_string(),
        ],
        datasets: vec![ChartDataset {
            label: "Assets".to_string(),
            data: vec![42.0, 36.0, 68.0, 125.0],
            background_colors: Some(vec![
                "#4299E1FF".to_string(),
                "#2FB344FF".to_string(),
                "#F59F00FF".to_string(),
                "#D63939FF".to_string(),
            ]),
            border_colors: Some(vec![
                "#4299E1".to_string(),
                "#2FB344".to_string(),
                "#F59F00".to_string(),
                "#D63939".to_string(),
            ]),
        }],
    };

    // Sample data that uses the formatting utils
    let sample_date = "2023-04-15T14:32:18Z";
    let formatted_date = format_date(sample_date);
    let severity = "high";
    let formatted_severity = format_severity(severity);
    let severity_css_class = severity_class(severity);
    let long_text = "This is a very long description that should be truncated for display purposes";
    let truncated_text = truncate(long_text, 20);

    view! {
        <div>
            <div class="page-header">
                <h1 class="page-title">"Dashboard"</h1>
            </div>

            <div class="grid grid-3">
                <div class="card">
                    <h2>"Asset Summary"</h2>
                    <div class="stat">
                        <span class="stat-value">"125"</span>
                        <span class="stat-label">"Total Assets"</span>
                    </div>
                </div>

                <div class="card">
                    <h2>"Vulnerability Summary"</h2>
                    <div class="stat">
                        <span class="stat-value">"42"</span>
                        <span class="stat-label">"Open Vulnerabilities"</span>
                    </div>
                </div>

                <div class="card">
                    <h2>"Technology Summary"</h2>
                    <div class="stat">
                        <span class="stat-value">"18"</span>
                        <span class="stat-label">"Detected Technologies"</span>
                    </div>
                </div>
            </div>

            <div class="grid grid-2">
                <div class="card">
                    <h2>"Vulnerabilities by Severity"</h2>
                    <Chart
                        title="Vulnerabilities by Severity"
                        chart_type=ChartType::Pie
                        data=vulnerability_chart_data
                        height=250
                        show_legend=true
                    />
                </div>

                <div class="card">
                    <h2>"Assets by Type"</h2>
                    <Chart
                        title="Assets by Type"
                        chart_type=ChartType::Bar
                        data=asset_chart_data
                        height=250
                        show_legend=true
                    />
                </div>
            </div>

            <div class="card">
                <h2>"Recent Discoveries"</h2>
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Type"</th>
                            <th>"Asset"</th>
                            <th>"Timestamp"</th>
                            <th>"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Domain"</td>
                            <td>{truncate("example.com", 15)}</td>
                            <td>{formatted_date}</td>
                            <td><span class="badge badge-success">"Active"</span></td>
                        </tr>
                        <tr>
                            <td>"IP Address"</td>
                            <td>"192.168.1.1"</td>
                            <td>"2023-03-15 12:15:45"</td>
                            <td><span class="badge badge-success">"Active"</span></td>
                        </tr>
                        <tr>
                            <td>"Technology"</td>
                            <td>"WordPress 5.9.3"</td>
                            <td>"2023-03-14 18:22:10"</td>
                            <td><span class="badge badge-success">"Detected"</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="card">
                <h2>"Recent Vulnerabilities"</h2>
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Title"</th>
                            <th>"Asset"</th>
                            <th>"Severity"</th>
                            <th>"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>{truncated_text}</td>
                            <td>"example.com"</td>
                            <td><span class={"badge ".to_string() + &severity_css_class}>{formatted_severity}</span></td>
                            <td><span class="badge badge-danger">"Open"</span></td>
                        </tr>
                        <tr>
                            <td>"SSL Certificate Expiring Soon"</td>
                            <td>"api.example.com"</td>
                            <td><span class="badge severity-medium">"Medium"</span></td>
                            <td><span class="badge badge-warning">"In Progress"</span></td>
                        </tr>
                        <tr>
                            <td>"Open SSH Port"</td>
                            <td>"192.168.1.1"</td>
                            <td><span class="badge severity-low">"Low"</span></td>
                            <td><span class="badge badge-danger">"Open"</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
