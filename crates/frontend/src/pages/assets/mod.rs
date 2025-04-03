use crate::api::ApiClient;
use crate::components::ui::asset_card::{Asset, AssetCard};
use leptos::prelude::*;

#[component]
pub fn AssetsPage() -> impl IntoView {
    // Create API client
    let _api_client = ApiClient::new("http://localhost:8080/api".to_string());

    // Create signals for assets and loading state
    let (assets, _set_assets) = signal(vec![
        Asset {
            id: "1".to_string(),
            name: "example.com".to_string(),
            asset_type: "Domain".to_string(),
            status: "Active".to_string(),
            discovery_date: "2023-03-15".to_string(),
            vulnerabilities_count: 3,
        },
        Asset {
            id: "2".to_string(),
            name: "api.example.com".to_string(),
            asset_type: "Subdomain".to_string(),
            status: "Active".to_string(),
            discovery_date: "2023-03-16".to_string(),
            vulnerabilities_count: 1,
        },
        Asset {
            id: "3".to_string(),
            name: "192.168.1.1".to_string(),
            asset_type: "IP Address".to_string(),
            status: "Inactive".to_string(),
            discovery_date: "2023-03-14".to_string(),
            vulnerabilities_count: 0,
        },
        Asset {
            id: "4".to_string(),
            name: "example.org".to_string(),
            asset_type: "Domain".to_string(),
            status: "Active".to_string(),
            discovery_date: "2023-03-12".to_string(),
            vulnerabilities_count: 5,
        },
    ]);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (_selected_asset, set_selected_asset) = signal::<Option<String>>(None);

    // Fetch assets on mount
    let fetch_assets = move || {
        set_loading.set(true);
        set_error.set(None);

        // In a real app, we'd use the API client to fetch assets
        // For now, we're simulating an API call with a timeout
        set_loading.set(false);
    };

    // Call the fetch_assets function when the component mounts
    let _effect = Effect::new(move |_| {
        fetch_assets();
    });

    // Handler for adding a new asset
    let add_asset = move |_| {
        // In a real app, this would open a modal or navigate to a form
        log::info!("Adding new asset");

        // Example of using the API client to create a new asset
        // In a real app, we'd get this data from a form
        // let new_asset = NewAsset { name: "new.example.com".to_string(), ... };
    };

    // Component for showing the loading state
    let loading_view = move || {
        loading.get().then(|| {
            view! {
                <div class="loading-spinner">"Loading assets..."</div>
            }
        })
    };

    // Component for showing the assets grid
    let assets_grid = move || {
        (!loading.get()).then(|| {
            view! {
                <div class="asset-grid">
                    {move || assets.get().into_iter().map(|asset| {
                        let asset_id = asset.id.clone();
                        let set_selected = set_selected_asset;

                        let on_click = Callback::new(move |_| {
                            set_selected.set(Some(asset_id.clone()));
                            log::info!("Asset selected: {}", asset_id);
                        });

                        view! {
                            <AssetCard
                                id={asset.id.clone()}
                                name={asset.name}
                                asset_type={asset.asset_type}
                                status={asset.status}
                                discovery_date={asset.discovery_date}
                                vulnerabilities_count={asset.vulnerabilities_count}
                                on_click={on_click}
                            />
                        }
                    }).collect::<Vec<_>>()}
                </div>
            }
        })
    };

    view! {
        <div class="page-container">
            <div class="page-header">
                <h1 class="page-title">"Assets"</h1>
                <div class="page-actions">
                    <button class="btn btn-primary" on:click=add_asset>"Add New Asset"</button>
                    <button class="btn btn-secondary">"Import Assets"</button>
                </div>
            </div>

            <div class="filter-bar">
                <input type="text" placeholder="Search assets..." class="search-input" />
                <select class="filter-select">
                    <option>"All Types"</option>
                    <option>"Domain"</option>
                    <option>"Subdomain"</option>
                    <option>"IP Address"</option>
                </select>
                <select class="filter-select">
                    <option>"All Statuses"</option>
                    <option>"Active"</option>
                    <option>"Inactive"</option>
                </select>
                <button
                    class="btn btn-outline-primary"
                    on:click=move |_| fetch_assets()
                >
                    "Refresh"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert alert-danger">{err}</div>
            })}

            {loading_view}
            {assets_grid}
        </div>
    }
}
