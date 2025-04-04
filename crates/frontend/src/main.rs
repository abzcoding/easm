mod api;
mod components;
mod pages;
mod router;
mod utils;

use leptos::prelude::*;
use router::AppRouter;

fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    log::info!("EASM Frontend Starting...");

    // Mount the router
    mount_to_body(|| {
        view! {
            <ErrorBoundary
                fallback=|errors| view! {
                    <div class="error-boundary" style="padding: 20px; text-align: center; font-family: sans-serif;">
                        <h1>"Something went wrong!"</h1>
                        <p>"The application failed to initialize properly."</p>
                        <button
                            on:click=move |_| {
                                // Reload the page to try again
                                let window = web_sys::window().expect("no global window exists");
                                window.location().reload().expect("failed to reload");
                            }
                        >
                            "Reload Page"
                        </button>
                        <details>
                            <summary>"Error Details"</summary>
                            <pre style="max-width: 100%; overflow-x: auto;">
                                {format!("{:#?}", errors)}
                            </pre>
                        </details>
                    </div>
                }
            >
                <AppRouter />
            </ErrorBoundary>
        }
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
