use crate::components::layout::hooks::use_location;
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;

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

    view! {
        <nav class="navbar">
            <div class="navbar-container container">
                <A href="/" class="navbar-brand">
                    "EASM"
                </A>

                <div class="navbar-nav">
                    <A
                        href="/"
                        class=move || if is_active("/") && location.pathname.get() == "/" {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Dashboard"
                    </A>
                    <A
                        href="/assets"
                        class=move || if is_active("/assets") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Assets"
                    </A>
                    <A
                        href="/technologies"
                        class=move || if is_active("/technologies") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Technologies"
                    </A>
                    <A
                        href="/vulnerabilities"
                        class=move || if is_active("/vulnerabilities") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Vulnerabilities"
                    </A>
                    <A
                        href="/discovery"
                        class=move || if is_active("/discovery") {
                            "nav-link active"
                        } else {
                            "nav-link"
                        }
                    >
                        "Discovery"
                    </A>
                    <button class="btn btn-secondary" on:click=move |_| {
                        // Handle logout
                    }>
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
