use crate::api::ApiClient;
use crate::components::ui::asset_card::Asset;
use crate::components::ui::discovery_task::DiscoveryTaskForm;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn AssetDetailPage() -> impl IntoView {
    // Get the asset ID from the URL (in a real application)
    // For demo purposes, we'll use a fixed asset
    let asset_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap_or_default();
    
    // We'd fetch the asset details using the API client
    let _api_client = ApiClient::new("http://localhost:3000/api".to_string());
    
    // Mock asset data (in a real app, this would come from an API call)
    let (asset, _set_asset) = create_signal(Some(Asset {
        id: asset_id.to_string(),
        name: "example.com".to_string(),
        asset_type: "Domain".to_string(),
        status: "Active".to_string(),
        discovery_date: "2023-03-15".to_string(),
        vulnerabilities_count: 3,
    }));
    
    // State for showing the discovery task form
    let (show_discovery_task_form, set_show_discovery_task_form) = create_signal(false);
    
    // Handler for the "Add Discovery Task" button
    let add_discovery_task = move |_| {
        set_show_discovery_task_form.set(true);
    };
    
    // Handler for task creation success
    let on_task_created = move |_| {
        set_show_discovery_task_form.set(false);
        // Refresh asset data or tasks list
    };
    
    // Handler for cancelling task creation
    let on_task_cancel = move |_| {
        set_show_discovery_task_form.set(false);
    };
    
    view! {
        <div class="container mx-auto px-4 py-8">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold">Asset Details</h1>
                <div class="flex space-x-2">
                    <button
                        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                        on:click=add_discovery_task
                    >
                        "Add Discovery Task"
                    </button>
                    <a
                        href="/assets"
                        class="px-4 py-2 border border-gray-300 rounded hover:bg-gray-50"
                    >
                        "Back to Assets"
                    </a>
                </div>
            </div>
            
            <Show
                when=move || asset.get().is_some()
                fallback=|| view! { <div>"Loading asset details..."</div> }
            >
                {move || {
                    let asset = asset.get().unwrap();
                    view! {
                        <div class="bg-white shadow overflow-hidden sm:rounded-lg">
                            <div class="px-4 py-5 sm:px-6 bg-gray-50">
                                <h3 class="text-lg font-medium leading-6 text-gray-900">
                                    {asset.name}
                                </h3>
                                <p class="mt-1 max-w-2xl text-sm text-gray-500">
                                    {"ID: "}{asset.id}
                                </p>
                            </div>
                            <div class="border-t border-gray-200">
                                <dl>
                                    <div class="bg-white px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                                        <dt class="text-sm font-medium text-gray-500">Type</dt>
                                        <dd class="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
                                            {asset.asset_type}
                                        </dd>
                                    </div>
                                    <div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                                        <dt class="text-sm font-medium text-gray-500">Status</dt>
                                        <dd class="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
                                            {asset.status}
                                        </dd>
                                    </div>
                                    <div class="bg-white px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                                        <dt class="text-sm font-medium text-gray-500">Discovery Date</dt>
                                        <dd class="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
                                            {asset.discovery_date}
                                        </dd>
                                    </div>
                                    <div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                                        <dt class="text-sm font-medium text-gray-500">Vulnerabilities</dt>
                                        <dd class="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
                                            {asset.vulnerabilities_count}
                                        </dd>
                                    </div>
                                </dl>
                            </div>
                        </div>
                    }
                }}
            </Show>
            
            <div class="mt-8">
                <h2 class="text-xl font-bold mb-4">Recent Discovery Tasks</h2>
                
                // We'd fetch and display discovery tasks for this asset
                <div class="bg-white shadow overflow-hidden sm:rounded-lg p-4">
                    <p class="text-gray-500 italic">No discovery tasks found for this asset.</p>
                </div>
            </div>
            
            <Show when=move || show_discovery_task_form.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white rounded-lg shadow-xl max-w-lg w-full">
                        <DiscoveryTaskForm
                            asset_id=Some(asset_id)
                            organization_id=Some(Uuid::nil()) // In a real app, we'd use the asset's organization_id
                            on_success=on_task_created
                            on_cancel=on_task_cancel
                        />
                    </div>
                </div>
            </Show>
        </div>
    }
} 