use leptos::prelude::{provide_context, Children, Effect, ElementChild};
use leptos::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    location::RequestUrl,
    path, NavigateOptions,
};
use wasm_bindgen::JsValue;

use crate::components::layout::{AppLayout, AuthLayout};
use crate::pages::{
    assets::AssetsPage, auth::LoginPage, dashboard::DashboardPage, discovery::DiscoveryPage,
    not_found::NotFoundPage, technologies::TechnologiesPage, vulnerabilities::VulnerabilitiesPage,
};
use crate::utils::get_auth_token;

// Basic App Router
#[component]
pub fn AppRouter() -> impl IntoView {
    let fallback = move || view! { <NotFoundPage/> };

    // Get current URL
    let window = web_sys::window().expect("no global window exists");
    let location = window.location();
    let path = location.pathname().expect("no pathname exists");

    // Create and provide RequestUrl to router
    provide_context(RequestUrl::new(&path));
    log::info!("Set RequestUrl to path: {}", path);

    view! {
        <Stylesheet id="main" href="/style/output.css"/>
        <Router base="/">
            <Routes fallback>
                <Route path=path!("/") view=RedirectToLogin/>
                <Route path=path!("/login") view=LoginView/>
                // Protected routes
                <Route path=path!("/dashboard") view=move || view! { <RequireAuth><DashboardPage/></RequireAuth> }/>
                <Route path=path!("/app/assets") view=move || view! { <RequireAuth><AppLayout><AssetsPage/></AppLayout></RequireAuth> }/>
                <Route path=path!("/app/technologies") view=move || view! { <RequireAuth><AppLayout><TechnologiesPage/></AppLayout></RequireAuth> }/>
                <Route path=path!("/app/vulnerabilities") view=move || view! { <RequireAuth><AppLayout><VulnerabilitiesPage/></AppLayout></RequireAuth> }/>
                <Route path=path!("/app/discovery") view=move || view! { <RequireAuth><AppLayout><DiscoveryPage/></AppLayout></RequireAuth> }/>
            </Routes>
        </Router>
    }
}

// Authentication wrapper component
#[component]
fn RequireAuth(children: Children) -> impl IntoView {
    // Check for authentication on component mount
    let _effect = Effect::new(move |_| {
        // If no auth token exists, redirect to login
        if get_auth_token().is_none() {
            // Use window.location directly as a simple solution
            let window = web_sys::window().expect("no global window exists");
            let _ = window.location().replace("/login");
        }
    });

    // Render children if authenticated
    children()
}

// Route components
#[component]
fn RedirectToLogin() -> impl IntoView {
    let _effect = Effect::new(move |_| {
        let window = web_sys::window().expect("no global window exists");
        let _ = window.location().replace("/login");
    });

    view! { <div>"Redirecting to login..."</div> }
}

#[component]
fn LoginView() -> impl IntoView {
    view! { <AuthLayout><LoginPage/></AuthLayout> }
}
