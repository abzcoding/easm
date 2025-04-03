use leptos::prelude::*;
use web_sys::MouseEvent;

/// A reusable data table component
#[component]
pub fn DataTable<T: Clone + Send + Sync + 'static>(
    /// Headers for the table columns
    #[prop(into)]
    headers: Vec<String>,
    /// Data to be displayed in the table
    #[prop(into)]
    data: Vec<T>,
    /// Function to map data items to row cells
    #[prop(into)]
    row_mapper: Box<dyn Fn(&T) -> Vec<String> + Send + Sync>,
    /// Optional function for handling row clicks
    #[prop(into, optional)]
    on_row_click: Option<Callback<(T, MouseEvent)>>,
    /// Optional CSS class for the table
    #[prop(into, optional)]
    class: Option<String>,
    /// Flag to show loading state
    #[prop(default = false)]
    loading: bool,
    /// Message to display when there's no data
    #[prop(default = "No data available.".to_string())]
    empty_message: String,
) -> impl IntoView {
    // Create a signal for data
    let data = RwSignal::new(data);

    // Computed property to determine if data is empty
    let is_empty = move || data.get().is_empty() && !loading;

    // Format class string
    let table_class = move || format!("data-table {}", class.clone().unwrap_or_default());

    // Clone header values for closures
    let header0 = headers.get(0).cloned().unwrap_or_default();
    let header1 = headers.get(1).cloned().unwrap_or_default();
    let header2 = headers.get(2).cloned().unwrap_or_default();
    let header3 = headers.get(3).cloned().unwrap_or_default();
    let header4 = headers.get(4).cloned().unwrap_or_default();

    // Get headers length for colspan
    let headers_len = headers.len().to_string();

    view! {
        <div class="data-table-container">
            <table class={table_class}>
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
                            vec![view! {
                                <tr class="loading-row">
                                    <td colspan={headers_len.clone()} class="loading-cell">
                                        <div class="loading-spinner"></div>
                                        <span>"Loading..."</span>
                                    </td>
                                </tr>
                            }.into_any()].into_view()
                        } else if is_empty() {
                            vec![view! {
                                <tr class="empty-row">
                                    <td colspan={headers_len.clone()} class="empty-cell">
                                        {empty_message.clone()}
                                    </td>
                                </tr>
                            }.into_any()].into_view()
                        } else {
                            data.get().iter().map(|item| {
                                let item_clone = item.clone();
                                let row_cells = (row_mapper)(item);

                                let click_handler = move |evt: MouseEvent| {
                                    if let Some(callback) = on_row_click.as_ref() {
                                        callback.run((item_clone.clone(), evt));
                                    }
                                };

                                view! {
                                    <tr
                                        class="data-row"
                                        on:click={click_handler}
                                        class:clickable={on_row_click.is_some()}
                                    >
                                        <td>{row_cells.get(0).cloned().unwrap_or_default()}</td>
                                        <td>{row_cells.get(1).cloned().unwrap_or_default()}</td>
                                        <td>{row_cells.get(2).cloned().unwrap_or_default()}</td>
                                        <td>{row_cells.get(3).cloned().unwrap_or_default()}</td>
                                        <td>{row_cells.get(4).cloned().unwrap_or_default()}</td>
                                    </tr>
                                }.into_any()
                            }).collect::<Vec<_>>().into_view()
                        }
                    }}
                </tbody>
            </table>
        </div>
    }
}
