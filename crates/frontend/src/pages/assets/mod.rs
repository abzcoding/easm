use crate::api::{ApiClient, ApiError};
use crate::components::ui::asset_card::Asset;
use crate::utils::get_auth_token;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

// Request type for creating/updating assets
#[derive(Serialize, Debug, Clone)]
struct AssetRequest {
    asset_type: String,
    value: String,
    status: String,
    attributes: Option<serde_json::Value>,
}

// Response type from the assets API
#[derive(Deserialize, Debug, Clone)]
struct AssetResponse {
    id: String,
    asset_type: String,
    value: String,
    status: String,
    first_seen: String,
    last_seen: String,
    attributes: Option<serde_json::Value>,
    vulnerabilities_count: Option<i32>,
}

// Modal states
#[derive(Debug, Clone, PartialEq, Eq)]
enum ModalState {
    Closed,
    AddAsset,
    EditAsset(String),   // Asset ID being edited
    DeleteAsset(String), // Asset ID being deleted
}

#[allow(clippy::redundant_closure, non_snake_case, unused_braces)]
#[component]
pub fn AssetsPage() -> impl IntoView {
    // Create API client
    let api_client = create_rw_signal(ApiClient::new("http://localhost:8080/api".to_string()));

    // Set token if available
    if let Some(token) = get_auth_token() {
        api_client.update(|client| client.set_token(token));
    }

    // Create signals for assets and UI state
    let (assets, set_assets) = create_signal(Vec::<Asset>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (modal_state, set_modal_state) = create_signal(ModalState::Closed);

    // Form signals
    let (form_asset_type, set_form_asset_type) = create_signal(String::new());
    let (form_asset_value, set_form_asset_value) = create_signal(String::new());
    let (form_asset_status, set_form_asset_status) = create_signal("ACTIVE".to_string());

    // Search and filter signals
    let (search_query, set_search_query) = create_signal(String::new());
    let (filter_type, set_filter_type) = create_signal::<Option<String>>(None);
    let (filter_status, set_filter_status) = create_signal::<Option<String>>(None);

    // Define the signals for form values
    let (new_asset_name, set_new_asset_name) = create_signal("".to_string());
    let (new_asset_type, set_new_asset_type) = create_signal("".to_string());

    let (edit_asset_name, set_edit_asset_name) = create_signal("".to_string());
    let (edit_asset_type, set_edit_asset_type) = create_signal("".to_string());

    let (delete_asset_name, set_delete_asset_name) = create_signal("".to_string());

    // Function to fetch assets from the API
    let fetch_assets = move || {
        set_loading.set(true);
        set_error.set(None);

        let client = api_client.get().clone();
        spawn_local(async move {
            match client.get::<Vec<AssetResponse>>("/assets").await {
                Ok(response) => {
                    // Convert API response to Asset format
                    let mapped_assets: Vec<Asset> = response
                        .into_iter()
                        .map(|a| Asset {
                            id: a.id,
                            name: a.value.clone(),
                            asset_type: a.asset_type.clone(),
                            status: a.status.clone(),
                            discovery_date: a.first_seen,
                            vulnerabilities_count: a.vulnerabilities_count.unwrap_or(0),
                        })
                        .collect();

                    set_assets.set(mapped_assets);
                    set_loading.set(false);
                }
                Err(e) => {
                    let error_msg = match e {
                        ApiError::AuthError(_) => {
                            "Authentication error - please log in again".to_string()
                        }
                        ApiError::NetworkError(msg) => format!("Network error: {}", msg),
                        ApiError::ServerError(msg) => format!("Server error: {}", msg),
                        _ => "Failed to fetch assets".to_string(),
                    };

                    set_error.set(Some(error_msg));
                    set_loading.set(false);
                }
            }
        });
    };

    // Function to add an asset
    let add_asset = move || {
        let asset_type = form_asset_type.get();
        let asset_value = form_asset_value.get();
        let asset_status = form_asset_status.get();

        // Basic validation
        if asset_type.is_empty() || asset_value.is_empty() {
            set_error.set(Some("Asset type and value are required".to_string()));
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let asset_request = AssetRequest {
            asset_type,
            value: asset_value,
            status: asset_status,
            attributes: None,
        };

        let client = api_client.get().clone();
        spawn_local(async move {
            match client
                .post::<AssetResponse, _>("/assets", &asset_request)
                .await
            {
                Ok(_) => {
                    // Refresh the assets list and close the modal in the main thread
                    spawn_local(async move {
                        fetch_assets();
                        set_modal_state.set(ModalState::Closed);
                    });
                }
                Err(e) => {
                    let error_msg = match e {
                        ApiError::AuthError(_) => {
                            "Authentication error - please log in again".to_string()
                        }
                        ApiError::BadRequest(msg) => format!("Invalid input: {}", msg),
                        _ => "Failed to create asset".to_string(),
                    };

                    set_error.set(Some(error_msg));
                    set_loading.set(false);
                }
            }
        });
    };

    // Function to edit an asset
    let edit_asset = move |id: String| {
        let asset_type = form_asset_type.get();
        let asset_value = form_asset_value.get();
        let asset_status = form_asset_status.get();

        // Basic validation
        if asset_type.is_empty() || asset_value.is_empty() {
            set_error.set(Some("Asset type and value are required".to_string()));
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let asset_request = AssetRequest {
            asset_type,
            value: asset_value,
            status: asset_status,
            attributes: None,
        };

        let client = api_client.get().clone();
        spawn_local(async move {
            match client
                .put::<AssetResponse, _>(&format!("/assets/{}", id), &asset_request)
                .await
            {
                Ok(_) => {
                    // Refresh the assets list and close the modal in the main thread
                    spawn_local(async move {
                        fetch_assets();
                        set_modal_state.set(ModalState::Closed);
                    });
                }
                Err(e) => {
                    let error_msg = match e {
                        ApiError::AuthError(_) => {
                            "Authentication error - please log in again".to_string()
                        }
                        ApiError::BadRequest(msg) => format!("Invalid input: {}", msg),
                        _ => "Failed to update asset".to_string(),
                    };

                    set_error.set(Some(error_msg));
                    set_loading.set(false);
                }
            }
        });
    };

    // Function to delete an asset
    let delete_asset = move |id: String| {
        set_loading.set(true);
        set_error.set(None);

        let client = api_client.get().clone();
        spawn_local(async move {
            match client.delete::<()>(&format!("/assets/{}", id)).await {
                Ok(_) => {
                    // Refresh the assets list and close the modal in the main thread
                    spawn_local(async move {
                        fetch_assets();
                        set_modal_state.set(ModalState::Closed);
                    });
                }
                Err(e) => {
                    let error_msg = match e {
                        ApiError::AuthError(_) => {
                            "Authentication error - please log in again".to_string()
                        }
                        ApiError::BadRequest(msg) => format!("Invalid input: {}", msg),
                        _ => "Failed to delete asset".to_string(),
                    };

                    set_error.set(Some(error_msg));
                    set_loading.set(false);
                }
            }
        });
    };

    // Call the fetch_assets function when the component mounts
    create_effect(move |_| {
        fetch_assets();
    });

    // Loading indicator
    let loading_view = move || {
        loading.get().then(|| {
            view! {
                <div class="loading-overlay">
                    <div class="spinner"></div>
                </div>
            }
        })
    };

    // Assets grid component
    let assets_grid = move || {
        let filtered_assets = move || {
            let assets_list = assets.get();
            let query = search_query.get().to_lowercase();
            let type_filter = filter_type.get();
            let status_filter = filter_status.get();

            assets_list
                .into_iter()
                .filter(|asset| {
                    asset.name.to_lowercase().contains(&query)
                        && type_filter
                            .as_ref()
                            .map_or(true, |t| &asset.asset_type == t)
                        && status_filter.as_ref().map_or(true, |s| &asset.status == s)
                })
                .collect::<Vec<_>>()
        };

        view! {
            <div class="assets-grid">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Type"</th>
                            <th>"Value"</th>
                            <th>"Status"</th>
                            <th>"First Seen"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let assets = filtered_assets();
                            if assets.is_empty() {
                                view! {
                                    <tr>
                                        <td colspan="5" class="empty-state">
                                            "No assets found matching your criteria"
                                        </td>
                                    </tr>
                                }.into_any()
                            } else {
                                assets.into_iter().map(|asset| {
                                    let asset_id = asset.id.clone();
                                    let asset_clone = asset.clone();
                                    let name_for_delete = asset.name.clone();
                                    let asset_type_for_edit = asset.asset_type.clone();
                                    let name_for_edit = asset.name.clone();

                                    view! {
                                        <tr>
                                            <td>{asset.asset_type.clone()}</td>
                                            <td>{asset.name.clone()}</td>
                                            <td>
                                                <span class=format!("status-badge status-{}", asset.status.to_lowercase())>
                                                    {asset.status.clone()}
                                                </span>
                                            </td>
                                            <td>{asset.discovery_date}</td>
                                            <td class="actions-cell">
                                                <button
                                                    class="btn btn-icon btn-sm"
                                                    title="Edit asset"
                                                    on:click=move |_| {
                                                        let asset_id = asset_clone.id.clone();
                                                        set_edit_asset_name.set(name_for_edit.clone());
                                                        set_edit_asset_type.set(asset_type_for_edit.clone());
                                                        set_form_asset_value.set(name_for_edit.clone());
                                                        set_form_asset_type.set(asset_type_for_edit.clone());
                                                        set_modal_state.set(ModalState::EditAsset(asset_id));
                                                    }
                                                >
                                                    "✏️"
                                                </button>
                                                <button
                                                    class="btn btn-icon btn-sm btn-danger"
                                                    title="Delete asset"
                                                    on:click=move |_| {
                                                        set_delete_asset_name.set(name_for_delete.clone());
                                                        set_modal_state.set(ModalState::DeleteAsset(asset_id.clone()));
                                                    }
                                                >
                                                    "🗑️"
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
        }
    };

    // Main Component View
    view! {
        <div class="page-container">
            <div class="page-header">
                <h1 class="page-title">"Assets"</h1>
                <div class="page-actions">
                    <button class="btn btn-primary" on:click=move |_| {
                        set_new_asset_name.set("".to_string());
                        set_new_asset_type.set("".to_string());
                        set_form_asset_type.set("".to_string());
                        set_form_asset_value.set("".to_string());
                        set_modal_state.set(ModalState::AddAsset);
                    }>"Add New Asset"</button>
                    <button class="btn btn-secondary">"Import Assets"</button>
                </div>
            </div>

            <div class="filter-bar">
                <input
                    type="text"
                    placeholder="Search assets..."
                    class="search-input"
                    on:input=move |ev| {
                        let input = event_target::<HtmlInputElement>(&ev);
                        set_search_query.set(input.value());
                    }
                />
                <select
                    class="filter-select"
                    on:change=move |ev| {
                        let input = event_target::<HtmlInputElement>(&ev);
                        let value = input.value();
                        set_filter_type.set(if value == "all" { None } else { Some(value) });
                    }
                >
                    <option value="all">"All Types"</option>
                    <option value="DOMAIN">"Domain"</option>
                    <option value="IP_ADDRESS">"IP Address"</option>
                    <option value="WEB_APP">"Web Application"</option>
                    <option value="CERTIFICATE">"Certificate"</option>
                    <option value="CODE_REPO">"Code Repository"</option>
                </select>
                <select
                    class="filter-select"
                    on:change=move |ev| {
                        let input = event_target::<HtmlInputElement>(&ev);
                        let value = input.value();
                        set_filter_status.set(if value == "all" { None } else { Some(value) });
                    }
                >
                    <option value="all">"All Statuses"</option>
                    <option value="ACTIVE">"Active"</option>
                    <option value="INACTIVE">"Inactive"</option>
                    <option value="ARCHIVED">"Archived"</option>
                </select>
                <button
                    class="btn btn-outline-primary"
                    on:click=move |_| fetch_assets()
                >
                    "Refresh"
                </button>
            </div>

            <div>
                {move || {
                    error.get().map(|err| view! {
                        <div class="alert alert-danger">{err}</div>
                    })
                }}
            </div>

            <div>
                {loading_view()}
            </div>

            <div>
                {assets_grid()}
            </div>

            // Modals container using leptos Show component
            <div class="modals-container">
                <Show
                    when=move || matches!(modal_state.get(), ModalState::AddAsset)
                    fallback=|| ()
                >
                    <div class="modal-backdrop">
                        <div class="modal">
                            <div class="modal-header">
                                <h3>"Add Asset"</h3>
                                <button class="modal-close" on:click=move |_| set_modal_state.set(ModalState::Closed)>"×"</button>
                            </div>
                            <div class="modal-body">
                                <div class="error" style="display: none;"></div>
                                <div class="form-group">
                                    <label for="asset-name">"Asset Name"</label>
                                    <input
                                        id="asset-name"
                                        type="text"
                                        prop:value=new_asset_name.get()
                                        on:input=move |ev| {
                                            let val = event_target_value(&ev);
                                            let val_clone = val.clone();
                                            set_new_asset_name.set(val);
                                            set_form_asset_value.set(val_clone);
                                        }
                                    />
                                </div>
                                <div class="form-group">
                                    <label for="asset-type">"Asset Type"</label>
                                    <input
                                        id="asset-type"
                                        type="text"
                                        prop:value=new_asset_type.get()
                                        on:input=move |ev| {
                                            let val = event_target_value(&ev);
                                            let val_clone = val.clone();
                                            set_new_asset_type.set(val);
                                            set_form_asset_type.set(val_clone);
                                        }
                                    />
                                </div>
                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-secondary" on:click=move |_| set_modal_state.set(ModalState::Closed)>"Cancel"</button>
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| add_asset()
                                    disabled=loading
                                >
                                    "Save Asset"
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>

                <Show
                    when=move || matches!(modal_state.get(), ModalState::EditAsset(_))
                    fallback=|| ()
                >
                    {move || {
                        let id = match modal_state.get() {
                            ModalState::EditAsset(id) => id,
                            _ => String::new(),
                        };

                        view! {
                            <div class="modal-backdrop">
                                <div class="modal">
                                    <div class="modal-header">
                                        <h3>"Edit Asset"</h3>
                                        <button class="modal-close" on:click=move |_| set_modal_state.set(ModalState::Closed)>"×"</button>
                                    </div>
                                    <div class="modal-body">
                                        <div class="error" style="display: none;"></div>
                                        <div class="form-group">
                                            <label for="edit-asset-name">"Asset Name"</label>
                                            <input
                                                id="edit-asset-name"
                                                type="text"
                                                prop:value=edit_asset_name.get()
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    let val_clone = val.clone();
                                                    set_edit_asset_name.set(val);
                                                    set_form_asset_value.set(val_clone);
                                                }
                                            />
                                        </div>
                                        <div class="form-group">
                                            <label for="edit-asset-type">"Asset Type"</label>
                                            <input
                                                id="edit-asset-type"
                                                type="text"
                                                prop:value=edit_asset_type.get()
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    let val_clone = val.clone();
                                                    set_edit_asset_type.set(val);
                                                    set_form_asset_type.set(val_clone);
                                                }
                                            />
                                        </div>
                                    </div>
                                    <div class="modal-footer">
                                        <button class="btn btn-secondary" on:click=move |_| set_modal_state.set(ModalState::Closed)>"Cancel"</button>
                                        <button
                                            class="btn btn-primary"
                                            on:click=move |_| {
                                                let asset_id = id.clone();
                                                edit_asset(asset_id);
                                            }
                                            disabled=loading
                                        >
                                            "Save Asset"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }}
                </Show>

                <Show
                    when=move || matches!(modal_state.get(), ModalState::DeleteAsset(_))
                    fallback=|| ()
                >
                    {move || {
                        let id = match modal_state.get() {
                            ModalState::DeleteAsset(id) => id,
                            _ => String::new(),
                        };

                        view! {
                            <div class="modal-backdrop">
                                <div class="modal">
                                    <div class="modal-header">
                                        <h3>"Delete Asset"</h3>
                                        <button class="modal-close" on:click=move |_| set_modal_state.set(ModalState::Closed)>"×"</button>
                                    </div>
                                    <div class="modal-body">
                                        <div class="error" style="display: none;"></div>
                                        <p>"Are you sure you want to delete " <strong>{delete_asset_name.get()}</strong>"?"</p>
                                    </div>
                                    <div class="modal-footer">
                                        <button class="btn btn-secondary" on:click=move |_| set_modal_state.set(ModalState::Closed)>"Cancel"</button>
                                        <button
                                            class="btn btn-danger"
                                            on:click=move |_| {
                                                let asset_id = id.clone();
                                                delete_asset(asset_id);
                                            }
                                            disabled=loading
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}

// Helper methods to extract IDs from modal states
impl ModalState {
    fn into_edit_id(self) -> Option<String> {
        match self {
            ModalState::EditAsset(id) => Some(id),
            _ => None,
        }
    }

    fn into_delete_id(self) -> Option<String> {
        match self {
            ModalState::DeleteAsset(id) => Some(id),
            _ => None,
        }
    }
}

// Define modal_id methods to extract IDs more safely
fn get_edit_id(state: &ModalState) -> String {
    match state {
        ModalState::EditAsset(id) => id.clone(),
        _ => String::new(),
    }
}

fn get_delete_id(state: &ModalState) -> String {
    match state {
        ModalState::DeleteAsset(id) => id.clone(),
        _ => String::new(),
    }
}
