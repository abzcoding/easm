use leptos::prelude::*;

#[component]
pub fn DiscoveryPage() -> impl IntoView {
    view! {
        <div>
            <div class="page-header">
                <h1 class="page-title">"Discovery & Scanning"</h1>
                <button class="btn btn-primary">
                    "New Discovery Job"
                </button>
            </div>

            <div class="grid grid-2">
                <div class="card">
                    <h2>"Start New Discovery"</h2>
                    <form>
                        <div class="form-group">
                            <label class="form-label">"Starting Points"</label>
                            <textarea class="form-input" rows="3" placeholder="Enter domains or IP addresses, one per line"></textarea>
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Discovery Methods"</label>
                            <div class="checkbox-group">
                                <div class="checkbox-item">
                                    <input type="checkbox" id="dns_enum" checked />
                                    <label for="dns_enum">"DNS Enumeration"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input type="checkbox" id="cert_transparency" checked />
                                    <label for="cert_transparency">"Certificate Transparency"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input type="checkbox" id="port_scan" checked />
                                    <label for="port_scan">"Port Scanning"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input type="checkbox" id="web_crawl" checked />
                                    <label for="web_crawl">"Web Crawling"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input type="checkbox" id="tech_fingerprint" checked />
                                    <label for="tech_fingerprint">"Technology Fingerprinting"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input type="checkbox" id="vuln_scan" checked />
                                    <label for="vuln_scan">"Vulnerability Scanning"</label>
                                </div>
                            </div>
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Scan Depth"</label>
                            <select class="form-select">
                                <option value="1">"Light (1 level)"</option>
                                <option value="2" selected>"Normal (2 levels)"</option>
                                <option value="3">"Deep (3 levels)"</option>
                            </select>
                        </div>

                        <div class="form-group">
                            <button type="submit" class="btn btn-primary">"Start Discovery"</button>
                        </div>
                    </form>
                </div>

                <div class="card">
                    <h2>"Active Jobs"</h2>
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Job ID"</th>
                                <th>"Type"</th>
                                <th>"Status"</th>
                                <th>"Progress"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>"#JOB-1234"</td>
                                <td>"Full Discovery"</td>
                                <td><span class="badge badge-warning">"Running"</span></td>
                                <td>
                                    <div class="progress">
                                        <div class="progress-bar" style="width: 75%"></div>
                                    </div>
                                </td>
                                <td>
                                    <button class="btn btn-small btn-secondary">"View"</button>
                                    <button class="btn btn-small btn-secondary">"Cancel"</button>
                                </td>
                            </tr>
                            <tr>
                                <td>"#JOB-1233"</td>
                                <td>"Technology Scan"</td>
                                <td><span class="badge badge-warning">"Running"</span></td>
                                <td>
                                    <div class="progress">
                                        <div class="progress-bar" style="width: 45%"></div>
                                    </div>
                                </td>
                                <td>
                                    <button class="btn btn-small btn-secondary">"View"</button>
                                    <button class="btn btn-small btn-secondary">"Cancel"</button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="card">
                <h2>"Recent Discovery Jobs"</h2>
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Job ID"</th>
                            <th>"Type"</th>
                            <th>"Starting Points"</th>
                            <th>"Start Time"</th>
                            <th>"Duration"</th>
                            <th>"Findings"</th>
                            <th>"Status"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"#JOB-1232"</td>
                            <td>"Full Discovery"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-14 10:30:15"</td>
                            <td>"45m 12s"</td>
                            <td>"12 assets, 4 vulnerabilities"</td>
                            <td><span class="badge badge-success">"Completed"</span></td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Report"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"#JOB-1231"</td>
                            <td>"Vulnerability Scan"</td>
                            <td>"api.example.com"</td>
                            <td>"2023-03-13 14:22:08"</td>
                            <td>"12m 40s"</td>
                            <td>"3 vulnerabilities"</td>
                            <td><span class="badge badge-success">"Completed"</span></td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Report"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"#JOB-1230"</td>
                            <td>"DNS Enumeration"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-12 09:15:22"</td>
                            <td>"5m 30s"</td>
                            <td>"8 subdomains"</td>
                            <td><span class="badge badge-success">"Completed"</span></td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Report"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div class="pagination">
                    <button class="btn btn-secondary">"Previous"</button>
                    <span class="pagination-info">"Showing 1-3 of 24"</span>
                    <button class="btn btn-secondary">"Next"</button>
                </div>
            </div>
        </div>
    }
}
