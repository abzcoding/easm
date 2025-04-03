use leptos::prelude::*;
use leptos::*;

#[component]
pub fn VulnerabilitiesPage() -> impl IntoView {
    view! {
        <div>
            <div class="page-header">
                <h1 class="page-title">"Vulnerabilities"</h1>
                <div class="btn-group">
                    <button class="btn btn-secondary">"Correlate Vulnerabilities"</button>
                    <button class="btn btn-primary">"Add Vulnerability"</button>
                </div>
            </div>

            <div class="card">
                <div class="filter-bar">
                    <input type="text" class="form-input" placeholder="Search vulnerabilities..." />
                    <select class="form-select">
                        <option>"All Severities"</option>
                        <option>"Critical"</option>
                        <option>"High"</option>
                        <option>"Medium"</option>
                        <option>"Low"</option>
                    </select>
                    <select class="form-select">
                        <option>"All Statuses"</option>
                        <option>"Open"</option>
                        <option>"In Progress"</option>
                        <option>"Resolved"</option>
                    </select>
                    <button class="btn btn-secondary">"Filter"</button>
                </div>

                <table class="table">
                    <thead>
                        <tr>
                            <th>"Title"</th>
                            <th>"Asset"</th>
                            <th>"Severity"</th>
                            <th>"Status"</th>
                            <th>"First Detected"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Outdated WordPress Version"</td>
                            <td>"example.com"</td>
                            <td><span class="badge severity-high">"High"</span></td>
                            <td><span class="badge badge-danger">"Open"</span></td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Find Similar"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"SSL Certificate Expiring Soon"</td>
                            <td>"api.example.com"</td>
                            <td><span class="badge severity-medium">"Medium"</span></td>
                            <td><span class="badge badge-warning">"In Progress"</span></td>
                            <td>"2023-03-12"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Find Similar"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"Sensitive Data Exposure in API"</td>
                            <td>"api.example.com"</td>
                            <td><span class="badge severity-critical">"Critical"</span></td>
                            <td><span class="badge badge-danger">"Open"</span></td>
                            <td>"2023-03-15"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Find Similar"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div class="pagination">
                    <button class="btn btn-secondary">"Previous"</button>
                    <span class="pagination-info">"Showing 1-3 of 42"</span>
                    <button class="btn btn-secondary">"Next"</button>
                </div>
            </div>

            <div class="card">
                <h2>"Correlation Analysis"</h2>
                <p>"Correlation analysis helps identify related vulnerabilities across different assets. This can help identify systemic issues that may require broader remediation strategies."</p>

                <div class="correlation-graph">
                    <div id="correlation-chart">
                        <p class="chart-placeholder">"Interactive correlation graph will render here..."</p>
                    </div>
                </div>

                <h3>"Recent Correlations"</h3>
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Vulnerability Group"</th>
                            <th>"Assets Affected"</th>
                            <th>"Count"</th>
                            <th>"Highest Severity"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Outdated Software Versions"</td>
                            <td>"5 assets"</td>
                            <td>"8 vulnerabilities"</td>
                            <td><span class="badge severity-high">"High"</span></td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Group"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"Misconfigured Services"</td>
                            <td>"3 assets"</td>
                            <td>"4 vulnerabilities"</td>
                            <td><span class="badge severity-medium">"Medium"</span></td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Group"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
