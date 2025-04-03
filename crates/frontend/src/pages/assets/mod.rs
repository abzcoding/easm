use crate::components::ui::asset_card::{Asset, AssetCard};
use leptos::prelude::*;

#[component]
pub fn AssetsPage() -> impl IntoView {
    // In a real app, this would be fetched from API
    let assets = create_signal(vec![
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

    let selected_asset = create_signal::<Option<String>>(None);

    let handle_asset_click = create_callback(move |id: String| {
        selected_asset.set(Some(id));
        log::info!("Asset selected: {}", id);
        // In a real app, this would navigate to asset details or open a modal
    });

    view! {
        <div class="page-container">
            <div class="page-header">
                <h1 class="page-title">"Assets"</h1>
                <div class="page-actions">
                    <button class="btn btn-primary">"Add New Asset"</button>
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
            </div>

            <div class="asset-grid">
                {move || assets.get().into_iter().map(|asset| {
                    view! {
                        <AssetCard
                            id={asset.id.clone()}
                            name={asset.name}
                            asset_type={asset.asset_type}
                            status={asset.status}
                            discovery_date={asset.discovery_date}
                            vulnerabilities_count={asset.vulnerabilities_count}
                            on_click={handle_asset_click}
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
