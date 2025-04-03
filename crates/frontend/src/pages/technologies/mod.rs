use leptos::prelude::*;

#[component]
pub fn TechnologiesPage() -> impl IntoView {
    view! {
        <div>
            <div class="page-header">
                <h1 class="page-title">"Technologies"</h1>
                <button class="btn btn-primary">
                    "Scan for Technologies"
                </button>
            </div>

            <div class="card">
                <div class="filter-bar">
                    <input type="text" class="form-input" placeholder="Search technologies..." />
                    <select class="form-select">
                        <option>"All Categories"</option>
                        <option>"Web Servers"</option>
                        <option>"Frameworks"</option>
                        <option>"CMS"</option>
                        <option>"JavaScript Libraries"</option>
                        <option>"Analytics"</option>
                    </select>
                    <button class="btn btn-secondary">"Filter"</button>
                </div>

                <table class="table">
                    <thead>
                        <tr>
                            <th>"Name"</th>
                            <th>"Version"</th>
                            <th>"Category"</th>
                            <th>"Asset"</th>
                            <th>"First Detected"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Apache"</td>
                            <td>"2.4.51"</td>
                            <td>"Web Server"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Details"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"WordPress"</td>
                            <td>"5.9.3"</td>
                            <td>"CMS"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Details"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"jQuery"</td>
                            <td>"3.6.0"</td>
                            <td>"JavaScript Library"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Details"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"Bootstrap"</td>
                            <td>"5.1.3"</td>
                            <td>"CSS Framework"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Details"</button>
                            </td>
                        </tr>
                        <tr>
                            <td>"Google Analytics"</td>
                            <td>"UA-123456-7"</td>
                            <td>"Analytics"</td>
                            <td>"example.com"</td>
                            <td>"2023-03-10"</td>
                            <td>
                                <button class="btn btn-small btn-secondary">"View Details"</button>
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div class="pagination">
                    <button class="btn btn-secondary">"Previous"</button>
                    <span class="pagination-info">"Showing 1-5 of 23"</span>
                    <button class="btn btn-secondary">"Next"</button>
                </div>
            </div>

            <div class="card">
                <h2>"Technology Distribution"</h2>
                <div class="chart-container">
                    <div id="technology-chart">
                        <p class="chart-placeholder">"Chart will render here..."</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
