use leptos::prelude::*;
use web_sys::MouseEvent;

/// Props for the DataTable component
#[derive(Clone)]
pub struct DataTableProps<T: Clone + 'static> {
    /// Headers for the table columns
    pub headers: Vec<String>,
    /// Data to be displayed in the table
    pub data: Vec<T>,
    /// Function to map data items to row cells
    pub row_mapper: Box<dyn Fn(&T) -> Vec<View>>,
    /// Optional function for handling row clicks
    pub on_row_click: Option<Box<dyn Fn(&T, MouseEvent)>>,
    /// Optional CSS class for the table
    pub class: Option<String>,
    /// Flag to show loading state
    pub loading: bool,
    /// Message to display when there's no data
    pub empty_message: String,
}

/// A reusable data table component
#[component]
pub fn DataTable<T: Clone + 'static>(
    /// Table props
    #[prop(into)]
    props: DataTableProps<T>,
) -> impl IntoView {
    // Computed property to determine if data is empty
    let is_empty = move || props.data.is_empty() && !props.loading;

    view! {
        <div class="data-table-container">
            <table class={move || format!("data-table {}", props.class.clone().unwrap_or_default())}>
                <thead>
                    <tr>
                        {props.headers.iter().map(|header| view! {
                            <th>{header}</th>
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        if props.loading {
                            view! {
                                <tr class="loading-row">
                                    <td colspan={props.headers.len().to_string()} class="loading-cell">
                                        <div class="loading-spinner"></div>
                                        <span>"Loading..."</span>
                                    </td>
                                </tr>
                            }.into_view()
                        } else if is_empty() {
                            view! {
                                <tr class="empty-row">
                                    <td colspan={props.headers.len().to_string()} class="empty-cell">
                                        {props.empty_message.clone()}
                                    </td>
                                </tr>
                            }.into_view()
                        } else {
                            props.data.iter().map(|item| {
                                let row_cells = (props.row_mapper)(item);
                                let item_clone = item.clone();
                                let on_click = props.on_row_click.as_ref().map(|handler| {
                                    let item = item_clone.clone();
                                    let handler = handler.clone();
                                    move |evt: MouseEvent| {
                                        handler(&item, evt);
                                    }
                                });

                                view! {
                                    <tr
                                        class="data-row"
                                        on:click={on_click}
                                        class:clickable={props.on_row_click.is_some()}
                                    >
                                        {row_cells}
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
