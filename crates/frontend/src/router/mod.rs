use leptos::*;
use leptos_meta::*;
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path,
};
use leptos::prelude::Effect;

use crate::components::layout::{AppLayout, AuthLayout};
use crate::pages::{
    assets::AssetsPage, auth::LoginPage, dashboard::DashboardPage, discovery::DiscoveryPage,
    not_found::NotFoundPage, technologies::TechnologiesPage, vulnerabilities::VulnerabilitiesPage,
};

#[component]
fn LoginView() -> impl IntoView {
    view! {
        <AuthLayout>
            <LoginPage/>
        </AuthLayout>
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    view! {
        <AppLayout>
            <Outlet/>
        </AppLayout>
    }
}

#[component]
fn NotFoundView() -> impl IntoView {
    view! {
        <NotFoundPage/>
    }
}

#[component]
fn DashboardView() -> impl IntoView {
    view! {
        <DashboardPage/>
    }
}

#[component]
fn AssetsView() -> impl IntoView {
    view! {
        <AssetsPage/>
    }
}

#[component]
fn TechnologiesView() -> impl IntoView {
    view! {
        <TechnologiesPage/>
    }
}

#[component]
fn VulnerabilityView() -> impl IntoView {
    view! {
        <VulnerabilitiesPage/>
    }
}

#[component]
fn DiscoveryView() -> impl IntoView {
    view! {
        <DiscoveryPage/>
    }
}

// Redirect to login route
#[component]
fn RedirectToLogin() -> impl IntoView {
    // Use window.location to redirect
    let _effect = Effect::new(move |_| {
        let window = web_sys::window().expect("no global window exists");
        let _ = window.location().replace("/login");
    });

    // Render nothing while redirecting
    view! { <div></div> }
}

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Stylesheet id="main" href="/style/output.css"/>
        <Router>
            <Routes fallback=|| view! { <NotFoundView/> }>
                // Default route redirects to login
                <Route path=path!("/") view=RedirectToLogin/>

                // Login page as a top-level route
                <Route path=path!("/login") view=LoginView/>

                // Dashboard as a top-level route
                <Route path=path!("/dashboard") view=DashboardView/>

                // Other app routes under parent route
                <ParentRoute path=path!("/app") view=MainLayout>
                    <Route path=path!("/assets") view=AssetsView/>
                    <Route path=path!("/technologies") view=TechnologiesView/>
                    <Route path=path!("/vulnerabilities") view=VulnerabilityView/>
                    <Route path=path!("/discovery") view=DiscoveryView/>
                </ParentRoute>

                <Route path=path!("*") view=NotFoundView/>
            </Routes>
        </Router>
    }
}
