use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::layout::{AppLayout, AuthLayout};
use crate::pages::{
    assets::AssetsPage, auth::LoginPage, dashboard::DashboardPage, discovery::DiscoveryPage,
    not_found::NotFoundPage, technologies::TechnologiesPage, vulnerabilities::VulnerabilitiesPage,
};
use crate::router::components::*;

// Wrapper component for AppLayout to use with routes
#[component]
fn AppLayoutWrapper() -> impl IntoView {
    view! {
        <AppLayout>
            <Outlet/>
        </AppLayout>
    }
}

#[component]
pub fn AppRouter() -> impl IntoView {
    // Provide the router to the app
    view! {
        <Stylesheet id="main" href="/style/output.css"/>
        <Router>
            <Routes>
                // Auth routes
                <Route path="login" view=|| view! { <AuthLayout><LoginPage/></AuthLayout> }/>

                // Protected app routes
                <Route path="" view=AppLayoutWrapper>
                    <Route path="/" view=|| view! { <DashboardPage/> }/>
                    <Route path="/assets" view=|| view! { <AssetsPage/> }/>
                    <Route path="/technologies" view=|| view! { <TechnologiesPage/> }/>
                    <Route path="/vulnerabilities" view=|| view! { <VulnerabilitiesPage/> }/>
                    <Route path="/discovery" view=|| view! { <DiscoveryPage/> }/>
                </Route>

                // 404 route
                <Route path="/*" view=|| view! { <NotFoundPage/> }/>
            </Routes>
        </Router>
    }
}
