mod api;
mod components;
mod pages;
mod router;
mod utils;

use leptos::prelude::*;
use leptos::*;
use router::AppRouter;

fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    log::info!("EASM Frontend Starting...");

    mount_to_body(|| {
        view! { <AppRouter /> }
    });
}

/// Simple component to display errors
#[component]
pub fn ErrorTemplate(#[prop(into)] error_message: String) -> impl IntoView {
    view! {
        <div class="error-container">
            <div class="error-content">
                <h1>"Error"</h1>
                <p>{error_message}</p>
            </div>
        </div>
    }
}
