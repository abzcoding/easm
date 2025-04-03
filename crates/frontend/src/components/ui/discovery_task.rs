use gloo_net::http::Request;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};

/// Discovery task type options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryTaskType {
    DnsEnumeration,
    PortScan,
    PortScanNaabu,
    WebAppScan,
    WebAppScanHttpx,
    CertificateTransparency,
    VulnerabilityScanNuclei,
}

/// Parameters for Nuclei vulnerability scanning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NucleiTaskParams {
    pub templates: Option<Vec<String>>,
    pub severity: Option<String>,
    pub rate_limit: Option<u32>,
    pub follow_redirects: Option<bool>,
    pub max_host_error: Option<u32>,
    pub timeout: Option<u32>,
}

/// Request body for creating a discovery task
#[derive(Debug, Serialize)]
struct CreateDiscoveryTaskRequest {
    organization_id: Uuid,
    asset_id: Option<Uuid>,
    target: Option<String>,
    task_type: DiscoveryTaskType,
    nuclei_params: Option<NucleiTaskParams>,
}

/// Discovery task form props
#[derive(Props, Clone, PartialEq)]
pub struct DiscoveryTaskFormProps {
    #[prop(optional)]
    pub asset_id: Option<Uuid>,
    #[prop(optional)]
    pub organization_id: Option<Uuid>,
    pub on_success: Callback<()>,
    pub on_cancel: Callback<()>,
}

/// Add discovery task component
#[component]
pub fn DiscoveryTaskForm(props: DiscoveryTaskFormProps) -> impl IntoView {
    let (task_type, set_task_type) = create_signal(DiscoveryTaskType::DnsEnumeration);
    let (target, set_target) = create_signal(String::new());
    let (is_nuclei, set_is_nuclei) = create_signal(false);
    let (templates, set_templates) = create_signal(String::new());
    let (severity, set_severity) = create_signal(String::from("medium,high,critical"));
    let (rate_limit, set_rate_limit) = create_signal(50u32);
    let (is_submitting, set_is_submitting) = create_signal(false);
    let (error_message, set_error_message) = create_signal(String::new());

    // Watch for changes to task_type to update the is_nuclei flag
    create_effect(move |_| {
        set_is_nuclei.update(|is_nuclei| {
            *is_nuclei = task_type.get() == DiscoveryTaskType::VulnerabilityScanNuclei;
        });
    });

    // Handler for task type selection
    let on_task_type_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let select = target.dyn_into::<HtmlSelectElement>().unwrap();
        let value = select.value();
        
        match value.as_str() {
            "DnsEnumeration" => set_task_type.set(DiscoveryTaskType::DnsEnumeration),
            "PortScan" => set_task_type.set(DiscoveryTaskType::PortScan),
            "PortScanNaabu" => set_task_type.set(DiscoveryTaskType::PortScanNaabu),
            "WebAppScan" => set_task_type.set(DiscoveryTaskType::WebAppScan),
            "WebAppScanHttpx" => set_task_type.set(DiscoveryTaskType::WebAppScanHttpx),
            "CertificateTransparency" => set_task_type.set(DiscoveryTaskType::CertificateTransparency),
            "VulnerabilityScanNuclei" => set_task_type.set(DiscoveryTaskType::VulnerabilityScanNuclei),
            _ => set_task_type.set(DiscoveryTaskType::DnsEnumeration),
        }
    };

    // Handler for target input
    let on_target_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input = target.dyn_into::<HtmlInputElement>().unwrap();
        set_target.set(input.value());
    };

    // Handler for templates input
    let on_templates_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input = target.dyn_into::<HtmlInputElement>().unwrap();
        set_templates.set(input.value());
    };

    // Handler for severity input
    let on_severity_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input = target.dyn_into::<HtmlInputElement>().unwrap();
        set_severity.set(input.value());
    };

    // Handler for rate limit input
    let on_rate_limit_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input = target.dyn_into::<HtmlInputElement>().unwrap();
        if let Ok(val) = input.value().parse::<u32>() {
            set_rate_limit.set(val);
        }
    };

    // Submit handler
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_is_submitting.set(true);
        set_error_message.set(String::new());

        // Validate required fields
        if props.asset_id.is_none() && target.get().is_empty() {
            set_error_message.set("Either select an asset or provide a target".into());
            set_is_submitting.set(false);
            return;
        }

        // Build the request
        let mut request = CreateDiscoveryTaskRequest {
            organization_id: props.organization_id.unwrap_or(Uuid::nil()),
            asset_id: props.asset_id,
            target: if target.get().is_empty() { None } else { Some(target.get()) },
            task_type: task_type.get(),
            nuclei_params: None,
        };

        // Add Nuclei parameters if applicable
        if is_nuclei.get() {
            let templates_vec = if templates.get().is_empty() {
                None
            } else {
                Some(
                    templates
                        .get()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>(),
                )
            };

            request.nuclei_params = Some(NucleiTaskParams {
                templates: templates_vec,
                severity: Some(severity.get()),
                rate_limit: Some(rate_limit.get()),
                follow_redirects: Some(true),
                max_host_error: None,
                timeout: None,
            });
        }

        // Get auth token from localStorage
        let token = web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .flatten()
            .and_then(|storage| storage.get_item("token").ok())
            .flatten();

        // Clone needed values for async block
        let on_success = props.on_success.clone();
        let set_error = set_error_message.clone();
        let set_submitting = set_is_submitting.clone();

        spawn_local(async move {
            // Submit the request
            let response = Request::post("/api/discovery-tasks")
                .header("Content-Type", "application/json")
                .header(
                    "Authorization",
                    &format!("Bearer {}", token.unwrap_or_default()),
                )
                .json(&request)
                .expect("Failed to serialize request")
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status() == 201 {
                        // Success
                        on_success.call(());
                    } else {
                        // Handle error
                        let error_text = resp.text().await.unwrap_or_else(|_| {
                            format!("Failed with status code: {}", resp.status())
                        });
                        set_error.set(error_text);
                    }
                }
                Err(err) => {
                    // Network error
                    set_error.set(format!("Network error: {}", err));
                }
            }
            
            set_submitting.set(false);
        });
    };

    view! {
        <div class="bg-white p-6 rounded-lg shadow-md max-w-lg mx-auto">
            <h2 class="text-2xl font-bold mb-6">Add Discovery Task</h2>
            
            <form on:submit=on_submit class="space-y-4">
                // Task Type selection
                <div>
                    <label for="task-type" class="block text-sm font-medium text-gray-700">
                        Task Type
                    </label>
                    <select
                        id="task-type"
                        on:change=on_task_type_change
                        class="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
                    >
                        <option value="DnsEnumeration">DNS Enumeration</option>
                        <option value="PortScan">Port Scan</option>
                        <option value="PortScanNaabu">Port Scan (Naabu)</option>
                        <option value="WebAppScan">Web App Scan</option>
                        <option value="WebAppScanHttpx">Web App Scan (Httpx)</option>
                        <option value="CertificateTransparency">Certificate Transparency</option>
                        <option value="VulnerabilityScanNuclei">Vulnerability Scan (Nuclei)</option>
                    </select>
                </div>
                
                // Only show target input if asset_id is not provided
                <Show when=move || props.asset_id.is_none()>
                    <div>
                        <label for="target" class="block text-sm font-medium text-gray-700">
                            Target (domain, IP, or URL)
                        </label>
                        <input
                            type="text"
                            id="target"
                            value=target
                            on:input=on_target_change
                            class="mt-1 focus:ring-blue-500 focus:border-blue-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                            placeholder="example.com"
                        />
                    </div>
                </Show>
                
                // Show Nuclei-specific parameters if applicable
                <Show when=move || is_nuclei.get()>
                    <div class="space-y-4 p-4 bg-gray-50 rounded-md">
                        <h3 class="font-medium">Nuclei Configuration</h3>
                        
                        <div>
                            <label for="templates" class="block text-sm font-medium text-gray-700">
                                Templates (comma-separated)
                            </label>
                            <input
                                type="text"
                                id="templates"
                                value=templates
                                on:input=on_templates_change
                                class="mt-1 focus:ring-blue-500 focus:border-blue-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                                placeholder="cves,exposures,vulnerabilities"
                            />
                            <p class="text-xs text-gray-500 mt-1">Leave empty to use default templates</p>
                        </div>
                        
                        <div>
                            <label for="severity" class="block text-sm font-medium text-gray-700">
                                Severity Levels
                            </label>
                            <input
                                type="text"
                                id="severity"
                                value=severity
                                on:input=on_severity_change
                                class="mt-1 focus:ring-blue-500 focus:border-blue-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                                placeholder="medium,high,critical"
                            />
                        </div>
                        
                        <div>
                            <label for="rate-limit" class="block text-sm font-medium text-gray-700">
                                Rate Limit (requests/second)
                            </label>
                            <input
                                type="number"
                                id="rate-limit"
                                value=rate_limit.get()
                                on:input=on_rate_limit_change
                                min="1"
                                max="1000"
                                class="mt-1 focus:ring-blue-500 focus:border-blue-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                            />
                        </div>
                    </div>
                </Show>
                
                // Error message
                <Show when=move || !error_message.get().is_empty()>
                    <div class="text-red-600 text-sm py-2">
                        {error_message.get()}
                    </div>
                </Show>
                
                // Submit and Cancel buttons
                <div class="flex justify-end space-x-3 pt-4">
                    <button
                        type="button"
                        class="py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        on:click=move |_| props.on_cancel.call(())
                        disabled=is_submitting.get()
                    >
                        Cancel
                    </button>
                    <button
                        type="submit"
                        class="py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        disabled=is_submitting.get()
                    >
                        {move || if is_submitting.get() { "Submitting..." } else { "Add Task" }}
                    </button>
                </div>
            </form>
        </div>
    }
} 