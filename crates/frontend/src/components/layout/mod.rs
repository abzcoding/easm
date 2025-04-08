use leptos::prelude::*;

use leptos_router::components::*;
use leptos_router::hooks::use_location;
use wasm_bindgen_futures::spawn_local;

use crate::utils::clear_auth_token;

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    view! {
        <div class="app-container">
            <Navbar />
            <main class="container">
                {children()}
            </main>
            <Footer />
        </div>
    }
}

#[component]
pub fn AuthLayout(children: Children) -> impl IntoView {
    view! {
        <div class="auth-container">
            <div class="auth-content">
                <div class="auth-logo">
                    <h1>"EASM"</h1>
                </div>
                {children()}
            </div>
        </div>
    }
}

#[component]
fn Navbar() -> impl IntoView {
    let location = use_location();

    let is_active = move |path: &str| location.pathname.get().starts_with(path);

    // Function to handle logout
    let handle_logout = move |_| {
        spawn_local(async move {
            // Clear the auth token
            if let Err(e) = clear_auth_token() {
                log::error!("Error clearing auth token: {}", e);
            }

            // Redirect to login page
            let window = web_sys::window().expect("no global window exists");
            let _ = window.location().replace("/login");
        });
    };

    view! {
        <nav class="navbar">
            <div class="navbar-container container">
                <A href="/dashboard" attr:class="navbar-brand">
                    "EASM"
                </A>

                <div class="navbar-nav">
                    <A
                        href="/dashboard"
                        attr:class=move || if is_active("/dashboard") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Dashboard"
                    </A>
                    <A
                        href="/app/assets"
                        attr:class=move || if is_active("/app/assets") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Assets"
                    </A>
                    <A
                        href="/app/technologies"
                        attr:class=move || if is_active("/app/technologies") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Technologies"
                    </A>
                    <A
                        href="/app/vulnerabilities"
                        attr:class=move || if is_active("/app/vulnerabilities") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Vulnerabilities"
                    </A>
                    <A
                        href="/app/discovery"
                        attr:class=move || if is_active("/app/discovery") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Discovery"
                    </A>
                    <button class="btn btn-secondary" on:click=handle_logout>
                        "Logout"
                    </button>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="container">
                <p class="footer-text">
                    "EASM - External Attack Surface Management © 2023"
                </p>
            </div>
        </footer>
    }
}
