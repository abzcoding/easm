use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub discovery_date: String,
    pub vulnerabilities_count: i32,
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
            callback.run(id.clone());
        }
    };

    // Clone the strings for closures first
    let name_clone = name.clone();
    let asset_type_clone = asset_type.clone();
    let status_clone = status.clone();
    let discovery_date_clone = discovery_date.clone();

    // Create copies for use in formatting closures
    let asset_type_for_class = asset_type_clone.clone();
    let status_for_class = status_clone.clone();

    let status_class = move || format!("status-badge status-{}", status_for_class.to_lowercase());
    let type_class = move || format!("type-badge type-{}", asset_type_for_class.to_lowercase());
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
                <span class={type_class}>{move || asset_type_clone.clone()}</span>
                <span class={status_class}>{move || status_clone.clone()}</span>
            </div>
            <div class="asset-card-content">
                <h3 class="asset-name">{move || name_clone.clone()}</h3>
                <div class="asset-info">
                    <div class="asset-date">
                        <span class="label">"Discovered:"</span>
                        <span class="value">{move || discovery_date_clone.clone()}</span>
                    </div>
                    <div class="asset-vuln">
                        <span class="label">"Vulnerabilities:"</span>
                        <span class={vuln_class}>{vulnerabilities_count}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
