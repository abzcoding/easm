use crate::components::ui::chart::{Chart, ChartData, ChartDataset, ChartType};
use crate::utils::{format_date, format_severity, severity_class, truncate};
use leptos::prelude::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    // Create chart data (assuming this data might be dynamic later)
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
            // Using slightly softer, modern colors inspired by examples
            background_colors: Some(vec![
                "rgba(199, 0, 57, 0.7)".to_string(),    // Crimson
                "rgba(255, 87, 51, 0.7)".to_string(),   // Orange Red
                "rgba(255, 195, 0, 0.7)".to_string(),   // Amber
                "rgba(52, 152, 219, 0.7)".to_string(),  // Belize Hole Blue
                "rgba(149, 165, 166, 0.7)".to_string(), // Silver
            ]),
            border_colors: Some(vec![
                "rgb(199, 0, 57)".to_string(),
                "rgb(255, 87, 51)".to_string(),
                "rgb(255, 195, 0)".to_string(),
                "rgb(52, 152, 219)".to_string(),
                "rgb(149, 165, 166)".to_string(),
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
                "rgba(52, 152, 219, 0.7)".to_string(), // Belize Hole Blue
                "rgba(46, 204, 113, 0.7)".to_string(), // Emerald Green
                "rgba(241, 196, 15, 0.7)".to_string(), // Sun Flower Yellow
                "rgba(231, 76, 60, 0.7)".to_string(),  // Alizarin Red
            ]),
            border_colors: Some(vec![
                "rgb(52, 152, 219)".to_string(),
                "rgb(46, 204, 113)".to_string(),
                "rgb(241, 196, 15)".to_string(),
                "rgb(231, 76, 60)".to_string(),
            ]),
        }],
    };

    // Sample data (keeping structure, just using utils)
    let sample_date = "2023-04-15T14:32:18Z";
    let formatted_date = format_date(sample_date);
    let severity_critical = "critical";
    let severity_medium = "medium";
    let severity_low = "low";
    let long_text = "Cross-Site Scripting (XSS) vulnerability found on login page allowing attackers to inject malicious scripts.";
    let truncated_text_title = truncate(long_text, 40); // Slightly longer truncation for title

    view! {
        // Use padding for overall spacing
        <div class="p-6 md:p-8 space-y-6">
            // Improved header styling
            <div class="page-header mb-6">
                <h1 class="text-3xl font-bold text-gray-800">"Dashboard Overview"</h1>
                <p class="text-gray-500">"Your central hub for asset and vulnerability insights."</p>
            </div>

            // Updated grid and card styles
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl">
                    <h2 class="text-lg font-semibold text-gray-700 mb-3">"Asset Summary"</h2>
                    <div class="stat flex items-baseline space-x-2">
                        <span class="stat-value text-4xl font-bold text-blue-600">"125"</span>
                        <span class="stat-label text-gray-500">"Total Assets"</span>
                    </div>
                </div>

                <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl">
                    <h2 class="text-lg font-semibold text-gray-700 mb-3">"Vulnerability Summary"</h2>
                    <div class="stat flex items-baseline space-x-2">
                        <span class="stat-value text-4xl font-bold text-red-600">"42"</span>
                        <span class="stat-label text-gray-500">"Open Vulnerabilities"</span>
                    </div>
                </div>

                <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl">
                    <h2 class="text-lg font-semibold text-gray-700 mb-3">"Technology Summary"</h2>
                    <div class="stat flex items-baseline space-x-2">
                        <span class="stat-value text-4xl font-bold text-green-600">"18"</span>
                        <span class="stat-label text-gray-500">"Detected Technologies"</span>
                    </div>
                </div>
            </div>

            // Grid for charts
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl">
                    <h2 class="text-lg font-semibold text-gray-700 mb-4">"Vulnerabilities by Severity"</h2>
                    // Ensure chart component adapts to container
                    <div class="h-[250px] flex justify-center items-center">
                        <Chart
                            title="" // Title is now part of the card header
                            chart_type=ChartType::Pie
                            data=vulnerability_chart_data
                            height=250 // Adjust as needed
                            show_legend=true
                        />
                    </div>
                </div>

                <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl">
                    <h2 class="text-lg font-semibold text-gray-700 mb-4">"Assets by Type"</h2>
                    <div class="h-[250px] flex justify-center items-center">
                        <Chart
                            title="" // Title is now part of the card header
                            chart_type=ChartType::Bar // Changed to Bar for variety, matching image 1
                            data=asset_chart_data
                            height=250 // Adjust as needed
                            show_legend=false // Often cleaner for bar charts
                        />
                    </div>
                </div>
            </div>

            // Recent Discoveries Table
            <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl overflow-x-auto">
                <h2 class="text-lg font-semibold text-gray-700 mb-4">"Recent Discoveries"</h2>
                <table class="table w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-gray-200">
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Type"</th>
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Asset"</th>
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Discovered"</th>
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Status"</th>
                        </tr>
                    </thead>
                    <tbody class="text-gray-700">
                        <tr class="hover:bg-gray-50 border-b border-gray-100">
                            <td class="py-3 px-4">"Domain"</td>
                            <td class="py-3 px-4">{truncate("example-long-domain-name.com", 25)}</td>
                            <td class="py-3 px-4">{formatted_date.clone()}</td>
                            <td class="py-3 px-4"><span class="badge bg-green-100 text-green-700 rounded-full px-3 py-1 text-xs font-medium">"Active"</span></td>
                        </tr>
                        <tr class="hover:bg-gray-50 border-b border-gray-100">
                            <td class="py-3 px-4">"IP Address"</td>
                            <td class="py-3 px-4">"198.51.100.42"</td>
                            <td class="py-3 px-4">{"2023-04-14 08:15:10"}</td>
                            <td class="py-3 px-4"><span class="badge bg-green-100 text-green-700 rounded-full px-3 py-1 text-xs font-medium">"Active"</span></td>
                        </tr>
                        <tr class="hover:bg-gray-50">
                            <td class="py-3 px-4">"Technology"</td>
                            <td class="py-3 px-4">"Nginx 1.23.1"</td>
                            <td class="py-3 px-4">{"2023-04-14 07:22:30"}</td>
                            <td class="py-3 px-4"><span class="badge bg-blue-100 text-blue-700 rounded-full px-3 py-1 text-xs font-medium">"Detected"</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>

            // Recent Vulnerabilities Table
            <div class="card bg-white shadow-lg rounded-xl p-6 transition hover:shadow-xl overflow-x-auto">
                <h2 class="text-lg font-semibold text-gray-700 mb-4">"Recent Vulnerabilities"</h2>
                <table class="table w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-gray-200">
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Title"</th>
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Asset"</th>
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Severity"</th>
                            <th class="py-3 px-4 font-semibold text-sm text-gray-600 uppercase">"Status"</th>
                        </tr>
                    </thead>
                    <tbody class="text-gray-700">
                        <tr class="hover:bg-gray-50 border-b border-gray-100">
                            <td class="py-3 px-4">{truncated_text_title}</td>
                            <td class="py-3 px-4">"login.example.com"</td>
                            // Using severity_class for dynamic styling based on severity
                            <td class="py-3 px-4"><span class={format!("badge rounded-full px-3 py-1 text-xs font-medium {}", severity_class(severity_critical))}>{format_severity(severity_critical)}</span></td>
                            <td class="py-3 px-4"><span class="badge bg-red-100 text-red-700 rounded-full px-3 py-1 text-xs font-medium">"Open"</span></td>
                        </tr>
                        <tr class="hover:bg-gray-50 border-b border-gray-100">
                            <td class="py-3 px-4">"TLS Certificate Expiring Soon"</td>
                            <td class="py-3 px-4">"api.example.com"</td>
                            <td class="py-3 px-4"><span class={format!("badge rounded-full px-3 py-1 text-xs font-medium {}", severity_class(severity_medium))}>{format_severity(severity_medium)}</span></td>
                            <td class="py-3 px-4"><span class="badge bg-yellow-100 text-yellow-700 rounded-full px-3 py-1 text-xs font-medium">"Review"</span></td>
                        </tr>
                        <tr class="hover:bg-gray-50">
                            <td class="py-3 px-4">"Open RDP Port (3389)"</td>
                            <td class="py-3 px-4">"198.51.100.42"</td>
                            <td class="py-3 px-4"><span class={format!("badge rounded-full px-3 py-1 text-xs font-medium {}", severity_class(severity_low))}>{format_severity(severity_low)}</span></td>
                            <td class="py-3 px-4"><span class="badge bg-red-100 text-red-700 rounded-full px-3 py-1 text-xs font-medium">"Open"</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
