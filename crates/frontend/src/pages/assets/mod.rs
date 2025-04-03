use leptos::prelude::*;
use leptos::*;

#[component]
pub fn AssetsPage() -> impl IntoView {
    view! {
        <div>
            <div class="page-header">
                <h1 class="page-title">"Assets"</h1>
                <div class="btn-group">
                    <button class="btn btn-secondary">"Import Assets"</button>
                    <button class="btn btn-primary">"Add Asset"</button>
                </div>
            </div>

            <div class="card">
                <div class="filter-bar">
                    <input type="text" class="form-input" placeholder="Search assets..." />
                    <select class="form-select">
                        <option>"All Types"</option>
                        <option>"Domain"</option>
                        <option>"IP Address"</option>
                        <option>"Web Application"</option>
                        <option>"Cloud Resource"</option>
                    </select>
                    <select class="form-select">
                        <option>"All Statuses"</option>
                        <option>"Active"</option>
                        <option>"Inactive"</option>
                        <option>"Archived"</option>
                    </select>
                    <button class="btn btn-secondary">"Filter"</button>
                </div>

                <table class="table">
                    <thead>
                        <tr>
                            <th>"Asset"</th>
                            <th>"Type"</th>
                            <th>"Status"</th>
                            <th>"Technologies"</th>
                            <th>"Vulnerabilities"</th>
                            <th>"Last Seen"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"example.com"</td>
                            <td>"Domain"</td>
                            <td><span class="badge badge-success">"Active"</span></td>
                            <td>"4"</td>
                            <td>"2"</td>
                            <td>"2023-03-15"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Scan"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"api.example.com"</td>
                            <td>"Domain"</td>
                            <td><span class="badge badge-success">"Active"</span></td>
                            <td>"3"</td>
                            <td>"3"</td>
                            <td>"2023-03-15"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Scan"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"192.168.1.1"</td>
                            <td>"IP Address"</td>
                            <td><span class="badge badge-success">"Active"</span></td>
                            <td>"2"</td>
                            <td>"1"</td>
                            <td>"2023-03-14"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Scan"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"storage.example.com"</td>
                            <td>"Cloud Resource"</td>
                            <td><span class="badge badge-warning">"Inactive"</span></td>
                            <td>"1"</td>
                            <td>"0"</td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View"</button>
                                <button class="btn btn-small btn-secondary">"Scan"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div class="pagination">
                    <button class="btn btn-secondary">"Previous"</button>
                    <span class="pagination-info">"Showing 1-4 of 125"</span>
                    <button class="btn btn-secondary">"Next"</button>
                </div>
            </div>

            <div class="grid grid-2">
                <div class="card">
                    <h2>"Asset Types Distribution"</h2>
                    <div class="chart-container">
                        <div id="asset-types-chart">
                            <p class="chart-placeholder">"Pie chart will render here..."</p>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h2>"Asset Risk Overview"</h2>
                    <div class="chart-container">
                        <div id="asset-risk-chart">
                            <p class="chart-placeholder">"Bar chart will render here..."</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
