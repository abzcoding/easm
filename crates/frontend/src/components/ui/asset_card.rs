use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct AssetCardProps {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub discovery_date: String,
    pub vulnerabilities_count: i32,
    pub on_click: Option<Callback<String>>,
}

#[component]
pub fn AssetCard(
    #[prop(into)] id: String,
    #[prop(into)] name: String,
    #[prop(into)] asset_type: String,
    #[prop(into)] status: String,
    #[prop(into)] discovery_date: String,
    #[prop(default = 0)] vulnerabilities_count: i32,
    #[prop(into, optional)] on_click: Option<Callback<String>>,
) -> impl IntoView {
    let handle_click = move |_| {
        if let Some(callback) = on_click.as_ref() {
            callback.call(id.clone());
        }
    };

    let status_class = move || format!("status-badge status-{}", status.to_lowercase());
    let type_class = move || format!("type-badge type-{}", asset_type.to_lowercase());
    let vuln_class = move || {
        if vulnerabilities_count == 0 {
            "vuln-count safe"
        } else if vulnerabilities_count < 3 {
            "vuln-count warning"
        } else {
            "vuln-count danger"
        }
    };

    view! {
        <div class="asset-card" on:click={handle_click}>
            <div class="asset-card-header">
                <h3 class="asset-name">{name}</h3>
                <span class={status_class}>{status}</span>
            </div>
            <div class="asset-card-body">
                <div class="asset-card-row">
                    <span class="label">"Type:"</span>
                    <span class={type_class}>{asset_type}</span>
                </div>
                <div class="asset-card-row">
                    <span class="label">"Discovery Date:"</span>
                    <span class="value">{discovery_date}</span>
                </div>
                <div class="asset-card-row">
                    <span class="label">"Vulnerabilities:"</span>
                    <span class={vuln_class}>{vulnerabilities_count.to_string()}</span>
                </div>
            </div>
        </div>
    }
}
