use leptos::prelude::{provide_context, Effect, ElementChild};
use leptos::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    location::RequestUrl,
    path,
};
use wasm_bindgen::JsValue;

use crate::components::layout::{AppLayout, AuthLayout};
use crate::pages::{
    assets::AssetsPage, auth::LoginPage, dashboard::DashboardPage, discovery::DiscoveryPage,
    not_found::NotFoundPage, technologies::TechnologiesPage, vulnerabilities::VulnerabilitiesPage,
};

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
                <Route path=path!("/dashboard") view=DashboardView/>
                <Route path=path!("/app/assets") view=AssetsView/>
                <Route path=path!("/app/technologies") view=TechnologiesView/>
                <Route path=path!("/app/vulnerabilities") view=VulnerabilityView/>
                <Route path=path!("/app/discovery") view=DiscoveryView/>
            </Routes>
        </Router>
    }
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

#[component]
fn DashboardView() -> impl IntoView {
    view! { <DashboardPage/> }
}

#[component]
fn AssetsView() -> impl IntoView {
    view! { <AppLayout><AssetsPage/></AppLayout> }
}

#[component]
fn TechnologiesView() -> impl IntoView {
    view! { <AppLayout><TechnologiesPage/></AppLayout> }
}

#[component]
fn VulnerabilityView() -> impl IntoView {
    view! { <AppLayout><VulnerabilitiesPage/></AppLayout> }
}

#[component]
fn DiscoveryView() -> impl IntoView {
    view! { <AppLayout><DiscoveryPage/></AppLayout> }
}
