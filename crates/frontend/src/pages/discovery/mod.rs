use crate::api::{ApiClient, ApiError};
use crate::utils::get_auth_token;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

// Request type for creating discovery jobs
#[derive(Serialize, Debug, Clone)]
struct DiscoveryJobRequest {
    job_type: String,
    targets: Vec<String>,
    configuration: DiscoveryConfiguration,
}

// Configuration for discovery jobs
#[derive(Serialize, Debug, Clone)]
struct DiscoveryConfiguration {
    depth: u8,
    methods: Vec<String>,
}

// Response type from the discovery API
#[derive(Deserialize, Debug, Clone)]
struct DiscoveryJobResponse {
    id: String,
    job_type: String,
    status: String,
    target: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    created_at: String,
    configuration: Option<serde_json::Value>,
}

// For active jobs table
#[derive(Debug, Clone)]
struct DiscoveryJob {
    id: String,
    job_type: String,
    status: String,
    target: String,
    started_at: String,
    progress: u8, // 0-100
}

#[component]
pub fn DiscoveryPage() -> impl IntoView {
    // Create API client
    let mut api_client_value = ApiClient::new("http://localhost:3000/api".to_string());

    // Set token if available
    if let Some(token) = get_auth_token() {
        api_client_value.set_token(token);
    }

    let api_client = StoredValue::new(api_client_value);

    // Discovery target input state
    let (targets, set_targets) = signal(String::new());

    // Discovery methods state (checkboxes)
    let (dns_enum, set_dns_enum) = signal(true);
    let (cert_transparency, set_cert_transparency) = signal(true);
    let (port_scan, set_port_scan) = signal(true);
    let (web_crawl, set_web_crawl) = signal(true);
    let (tech_fingerprint, set_tech_fingerprint) = signal(true);
    let (vuln_scan, set_vuln_scan) = signal(true);

    // Scan depth state
    let (scan_depth, set_scan_depth) = signal(2u8);

    // Loading, error and jobs state
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (active_jobs, set_active_jobs) = signal(Vec::<DiscoveryJob>::new());
    let (completed_jobs, set_completed_jobs) = signal(Vec::<DiscoveryJobResponse>::new());

    // Function to fetch all jobs - define this first to fix hoisting issue
    let fetch_jobs = Memo::new(move |_| {
        let client = api_client.get_value().clone();

        spawn_local(async move {
            // Fetch active jobs
            match client
                .get::<Vec<DiscoveryJobResponse>>("/discovery/jobs?status=RUNNING,PENDING")
                .await
            {
                Ok(response) => {
                    // Convert to active jobs
                    let active = response
                        .into_iter()
                        .map(|j| {
                            // Calculate mock progress
                            let progress = if j.status == "PENDING" {
                                0
                            } else {
                                // Generate a random progress for demo purposes
                                // In a real app, this would come from the API
                                (js_sys::Math::random() * 100.0) as u8
                            };

                            DiscoveryJob {
                                id: j.id,
                                job_type: j.job_type,
                                status: j.status,
                                target: j.target.unwrap_or_else(|| "Multiple targets".to_string()),
                                started_at: j.started_at.unwrap_or_else(|| "Pending".to_string()),
                                progress,
                            }
                        })
                        .collect::<Vec<_>>();

                    set_active_jobs.set(active);
                }
                Err(e) => {
                    log::error!("Failed to fetch active jobs: {:?}", e);
                }
            }

            // Fetch completed jobs
            match client
                .get::<Vec<DiscoveryJobResponse>>("/discovery/jobs?status=COMPLETED,FAILED")
                .await
            {
                Ok(response) => {
                    set_completed_jobs.set(response);
                }
                Err(e) => {
                    log::error!("Failed to fetch completed jobs: {:?}", e);
                }
            }
        });
    });

    // Separate implementation from closure
    let handle_discovery_submit =
        move |targets_value: String, scan_depth_val: u8, methods: Vec<String>| {
            let client = api_client.get_value().clone();

            // Validation already done in the closure
            set_loading.set(true);
            set_error.set(None);

            // Create job request
            let job_request = DiscoveryJobRequest {
                job_type: "FULL_DISCOVERY".to_string(),
                targets: targets_value
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                configuration: DiscoveryConfiguration {
                    depth: scan_depth_val,
                    methods,
                },
            };

            // Send the job request
            spawn_local(async move {
                match client
                    .post::<DiscoveryJobResponse, _>("/discovery/jobs", &job_request)
                    .await
                {
                    Ok(response) => {
                        // Add job to active jobs (simulate progress as 0%)
                        set_active_jobs.update(|jobs| {
                            jobs.push(DiscoveryJob {
                                id: response.id.clone(),
                                job_type: response.job_type.clone(),
                                status: response.status.clone(),
                                target: response
                                    .target
                                    .unwrap_or_else(|| "Multiple targets".to_string()),
                                started_at: response
                                    .started_at
                                    .unwrap_or_else(|| "Pending".to_string()),
                                progress: 0,
                            });
                        });

                        // Clear the form
                        set_targets.set(String::new());
                        set_loading.set(false);

                        // Refresh jobs by triggering the memo
                        fetch_jobs.get();
                    }
                    Err(e) => {
                        let error_msg = match e {
                            ApiError::AuthError(_) => {
                                "Authentication error - please log in again".to_string()
                            }
                            ApiError::BadRequest(msg) => format!("Invalid request: {}", msg),
                            ApiError::ServerError(msg) => format!("Server error: {}", msg),
                            _ => "Failed to start discovery job".to_string(),
                        };

                        set_error.set(Some(error_msg));
                        set_loading.set(false);
                    }
                }
            });
        };

    // Function to start a new discovery job
    let start_discovery = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        let targets_value = targets.get();
        let dns_enum_value = dns_enum.get();
        let cert_transparency_value = cert_transparency.get();
        let port_scan_value = port_scan.get();
        let web_crawl_value = web_crawl.get();
        let tech_fingerprint_value = tech_fingerprint.get();
        let vuln_scan_value = vuln_scan.get();
        let scan_depth_value = scan_depth.get();

        // Validation
        if targets_value.trim().is_empty() {
            set_error.set(Some("Please enter at least one target".to_string()));
            return;
        }

        // Prepare targets list (for validation only)
        let targets_list: Vec<String> = targets_value
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if targets_list.is_empty() {
            set_error.set(Some("Please enter at least one target".to_string()));
            return;
        }

        // Collect selected methods
        let mut methods = Vec::new();
        if dns_enum_value {
            methods.push("DNS_ENUM".to_string());
        }
        if cert_transparency_value {
            methods.push("CERT_SCAN".to_string());
        }
        if port_scan_value {
            methods.push("PORT_SCAN".to_string());
        }
        if web_crawl_value {
            methods.push("WEB_CRAWL".to_string());
        }
        if tech_fingerprint_value {
            methods.push("TECH_DETECT".to_string());
        }
        if vuln_scan_value {
            methods.push("VULN_SCAN".to_string());
        }

        if methods.is_empty() {
            set_error.set(Some(
                "Please select at least one discovery method".to_string(),
            ));
            return;
        }

        // Call our implementation with cloned values
        handle_discovery_submit(targets_value, scan_depth_value, methods);
    };

    // Function to cancel a job
    let cancel_job = move |job_id: String| {
        let client = api_client.get_value().clone();

        spawn_local(async move {
            match client
                .post::<(), _>(&format!("/discovery/jobs/{}/cancel", job_id), &())
                .await
            {
                Ok(_) => {
                    // Refresh jobs by getting the memo value
                    fetch_jobs.get();
                }
                Err(e) => {
                    log::error!("Failed to cancel job: {:?}", e);
                }
            }
        });
    };

    // Fetch jobs on mount and set up periodic refresh
    Effect::new(move |_| {
        // Initial fetch
        fetch_jobs.get();

        // Set up a timer to refresh jobs every 10 seconds
        let window = web_sys::window().expect("no global window exists");
        let callback = Closure::wrap(Box::new(move || {
            fetch_jobs.get();
        }) as Box<dyn FnMut()>);

        window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                10000, // 10 seconds
            )
            .expect("failed to set interval");

        // Prevent the closure from being dropped
        callback.forget();
    });

    view! {
        <div>
            <div class="page-header">
                <h1 class="page-title">"Discovery & Scanning"</h1>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_targets.set("".to_string())
                >
                    "New Discovery Job"
                </button>
            </div>

            <div class="grid grid-2">
                <div class="card">
                    <h2>"Start New Discovery"</h2>

                    {move || error.get().map(|err| view! {
                        <div class="alert alert-danger">{err}</div>
                    })}

                    <form on:submit=start_discovery>
                        <div class="form-group">
                            <label class="form-label">"Starting Points"</label>
                            <textarea
                                class="form-input"
                                rows="3"
                                placeholder="Enter domains or IP addresses, one per line"
                                prop:value=targets.get()
                                on:input=move |ev| {
                                    let input = event_target::<HtmlInputElement>(&ev);
                                    set_targets.set(input.value());
                                }
                            ></textarea>
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Discovery Methods"</label>
                            <div class="checkbox-group">
                                <div class="checkbox-item">
                                    <input
                                        type="checkbox"
                                        id="dns_enum"
                                        checked=dns_enum.get()
                                        on:change=move |ev| {
                                            let input = event_target::<HtmlInputElement>(&ev);
                                            set_dns_enum.set(input.checked());
                                        }
                                    />
                                    <label for="dns_enum">"DNS Enumeration"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input
                                        type="checkbox"
                                        id="cert_transparency"
                                        checked=cert_transparency.get()
                                        on:change=move |ev| {
                                            let input = event_target::<HtmlInputElement>(&ev);
                                            set_cert_transparency.set(input.checked());
                                        }
                                    />
                                    <label for="cert_transparency">"Certificate Transparency"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input
                                        type="checkbox"
                                        id="port_scan"
                                        checked=port_scan.get()
                                        on:change=move |ev| {
                                            let input = event_target::<HtmlInputElement>(&ev);
                                            set_port_scan.set(input.checked());
                                        }
                                    />
                                    <label for="port_scan">"Port Scanning"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input
                                        type="checkbox"
                                        id="web_crawl"
                                        checked=web_crawl.get()
                                        on:change=move |ev| {
                                            let input = event_target::<HtmlInputElement>(&ev);
                                            set_web_crawl.set(input.checked());
                                        }
                                    />
                                    <label for="web_crawl">"Web Crawling"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input
                                        type="checkbox"
                                        id="tech_fingerprint"
                                        checked=tech_fingerprint.get()
                                        on:change=move |ev| {
                                            let input = event_target::<HtmlInputElement>(&ev);
                                            set_tech_fingerprint.set(input.checked());
                                        }
                                    />
                                    <label for="tech_fingerprint">"Technology Fingerprinting"</label>
                                </div>
                                <div class="checkbox-item">
                                    <input
                                        type="checkbox"
                                        id="vuln_scan"
                                        checked=vuln_scan.get()
                                        on:change=move |ev| {
                                            let input = event_target::<HtmlInputElement>(&ev);
                                            set_vuln_scan.set(input.checked());
                                        }
                                    />
                                    <label for="vuln_scan">"Vulnerability Scanning"</label>
                                </div>
                            </div>
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Scan Depth"</label>
                            <select
                                class="form-select"
                                on:change=move |ev| {
                                    let input = event_target::<HtmlInputElement>(&ev);
                                    set_scan_depth.set(input.value().parse::<u8>().unwrap_or(2));
                                }
                            >
                                <option value="1">"Light (1 level)"</option>
                                <option value="2" selected=scan_depth.get() == 2>"Normal (2 levels)"</option>
                                <option value="3">"Deep (3 levels)"</option>
                            </select>
                        </div>

                        <div class="form-group">
                            <button
                                type="submit"
                                class="btn btn-primary"
                                disabled=loading.get()
                            >
                                {move || if loading.get() { "Starting..." } else { "Start Discovery" }}
                            </button>
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
                            {move || {
                                let jobs = active_jobs.get();
                                if jobs.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="5" style="text-align: center">"No active jobs"</td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    jobs.into_iter().map(|job| {
                                        let job_id = job.id.clone();
                                        let status_str = job.status.clone();
                                        let job_type = job.job_type.clone();
                                        let progress = job.progress;
                                        let cancel_job = cancel_job.clone();
                                        let status_display = if status_str == "RUNNING" { "warning" } else { "info" };

                                        view! {
                                            <tr>
                                                <td>{"#"}{job_id.chars().take(8).collect::<String>()}</td>
                                                <td>{job_type}</td>
                                                <td><span class=format!("badge badge-{}", status_display)>{status_str}</span></td>
                                                <td>
                                                    <div class="progress">
                                                        <div class="progress-bar" style=format!("width: {}%", progress)></div>
                                                    </div>
                                                </td>
                                                <td>
                                                    <button class="btn btn-small btn-secondary">"View"</button>
                                                    <button
                                                        class="btn btn-small btn-secondary"
                                                        on:click=move |_| {
                                                            let job_id = job_id.clone();
                                                            cancel_job(job_id);
                                                        }
                                                    >
                                                        "Cancel"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
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
                        {move || {
                            let jobs = completed_jobs.get();
                            if jobs.is_empty() {
                                view! {
                                    <tr>
                                        <td colspan="8" style="text-align: center">"No completed jobs yet"</td>
                                    </tr>
                                }.into_any()
                            } else {
                                jobs.into_iter().take(5).map(|job| {
                                    // Calculate duration (mock for now)
                                    let duration = "45m 12s";

                                    // Format findings (mock for now)
                                    let findings = "12 assets, 4 vulnerabilities";

                                    // Clone values to avoid ownership issues
                                    let job_id = job.id.clone();
                                    let job_type = job.job_type.clone();
                                    let target = job.target.clone().unwrap_or_else(|| "Multiple targets".to_string());
                                    let started_at = job.started_at.clone().unwrap_or_else(|| "-".to_string());
                                    let status = job.status.clone();
                                    let status_display = if status == "COMPLETED" { "success" } else { "danger" };

                                    view! {
                                        <tr>
                                            <td>{"#"}{job_id.chars().take(8).collect::<String>()}</td>
                                            <td>{job_type}</td>
                                            <td>{target}</td>
                                            <td>{started_at}</td>
                                            <td>{duration}</td>
                                            <td>{findings}</td>
                                            <td><span class=format!("badge badge-{}", status_display)>{status}</span></td>
                                            <td>
                                                <button class="btn btn-small btn-secondary">"View Report"</button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </tbody>
                </table>

                <div class="pagination">
                    <button class="btn btn-secondary">"Previous"</button>
                    <span class="pagination-info">"Showing 1-5 of " {move || completed_jobs.get().len()}</span>
                    <button class="btn btn-secondary">"Next"</button>
                </div>
            </div>
        </div>
    }
}
