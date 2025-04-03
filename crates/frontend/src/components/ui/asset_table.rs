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

    // Clone header values for closures
    let header0 = headers[0].clone();
    let header1 = headers[1].clone();
    let header2 = headers[2].clone();
    let header3 = headers[3].clone();
    let header4 = headers[4].clone();

    view! {
        <div class="asset-table-wrapper">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th>{move || header0.clone()}</th>
                        <th>{move || header1.clone()}</th>
                        <th>{move || header2.clone()}</th>
                        <th>{move || header3.clone()}</th>
                        <th>{move || header4.clone()}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        if loading {
                            let loading_row = view! {
                                <tr class="loading-row">
                                    <td colspan="5" class="loading-cell">
                                        <div class="loading-spinner"></div>
                                        <span>"Loading assets..."</span>
                                    </td>
                                </tr>
                            };
                            vec![loading_row].into_view()
                        } else if assets.get().is_empty() {
                            let empty_row = view! {
                                <tr class="empty-row">
                                    <td colspan="5" class="empty-cell">
                                        {empty_message.clone()}
                                    </td>
                                </tr>
                            };
                            vec![empty_row].into_view()
                        } else {
                            assets.get().iter().map(|asset| {
                                let asset_clone = asset.clone();

                                // Create a closure that will either call the callback or do nothing
                                let on_click = move |evt: MouseEvent| {
                                    if let Some(callback) = on_asset_click.as_ref() {
                                        callback.run((asset_clone.clone(), evt));
                                    }
                                };

                                // Clone string values for closures
                                let name = asset.name.clone();
                                let asset_type = asset.asset_type.clone();
                                let status = asset.status.clone();
                                let discovery_date = asset.discovery_date.clone();
                                let vuln_count = asset.vulnerabilities_count.to_string();

                                // Create class strings for formatting
                                let asset_type_class = format!("badge asset-type-{}", asset_type.to_lowercase());
                                let status_class = format!("badge status-{}", status.to_lowercase());

                                view! {
                                    <tr class="asset-row" class:clickable={on_asset_click.is_some()} on:click={on_click}>
                                        <td>{move || name.clone()}</td>
                                        <td><span class={asset_type_class}>{move || asset_type.clone()}</span></td>
                                        <td><span class={status_class}>{move || status.clone()}</span></td>
                                        <td>{move || discovery_date.clone()}</td>
                                        <td><span class="vuln-count">{move || vuln_count.clone()}</span></td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>().into_view()
                        }
                    }}
                </tbody>
            </table>
        </div>
    }
}
