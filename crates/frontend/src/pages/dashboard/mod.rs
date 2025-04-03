use leptos::prelude::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
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
                            <td>"example.com"</td>
                            <td>"2023-03-15 14:32:18"</td>
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
                            <td>"Outdated WordPress Version"</td>
                            <td>"example.com"</td>
                            <td><span class="badge severity-high">"High"</span></td>
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
