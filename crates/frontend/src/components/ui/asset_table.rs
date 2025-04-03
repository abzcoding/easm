use crate::components::ui::asset_card::Asset;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn AssetTable(
    #[prop(into)] assets: Signal<Vec<Asset>>,
    #[prop(default = false)] loading: bool,
    #[prop(default = "No assets found.".to_string())] empty_message: String,
    #[prop(into, optional)] on_asset_click: Option<Callback<(Asset, MouseEvent)>>,
) -> impl IntoView {
    // Define headers
    let headers = vec![
        "Name".to_string(),
        "Type".to_string(),
        "Status".to_string(),
        "Discovery Date".to_string(),
        "Vulnerabilities".to_string(),
    ];

    // Create table rows
    let asset_rows = move || {
        assets.get().iter().map(|asset| {
            let asset_clone = asset.clone();
            let on_click = on_asset_click.clone().map(|callback| {
                let asset = asset_clone.clone();
                move |evt: MouseEvent| {
                    callback.call((asset.clone(), evt));
                }
            });

            view! {
                <tr class="asset-row" class:clickable={on_asset_click.is_some()} on:click={on_click}>
                    <td>{&asset.name}</td>
                    <td><span class=format!("badge asset-type-{}", asset.asset_type.to_lowercase())>{&asset.asset_type}</span></td>
                    <td><span class=format!("badge status-{}", asset.status.to_lowercase())>{&asset.status}</span></td>
                    <td>{&asset.discovery_date}</td>
                    <td><span class="vuln-count">{asset.vulnerabilities_count.to_string()}</span></td>
                </tr>
            }
        }).collect::<Vec<_>>()
    };

    view! {
        <div class="asset-table-wrapper">
            <table class="asset-table">
                <thead>
                    <tr>
                        {headers.iter().map(|header| view! { <th>{header}</th> }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        if loading {
                            view! {
                                <tr class="loading-row">
                                    <td colspan="5" class="loading-cell">
                                        <div class="loading-spinner"></div>
                                        <span>"Loading assets..."</span>
                                    </td>
                                </tr>
                            }
                        } else if assets.get().is_empty() {
                            view! {
                                <tr class="empty-row">
                                    <td colspan="5" class="empty-cell">
                                        {empty_message.clone()}
                                    </td>
                                </tr>
                            }
                        } else {
                            asset_rows().into_view()
                        }
                    }}
                </tbody>
            </table>
        </div>
    }
}
