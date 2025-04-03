use leptos::prelude::*;
use leptos::*;
use leptos_router::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="not-found">
            <div class="not-found-container">
                <h1>"404"</h1>
                <h2>"Page Not Found"</h2>
                <p>"The page you are looking for does not exist or has been moved."</p>
                <A href="/" class="btn btn-primary">"Go to Dashboard"</A>
            </div>
        </div>
    }
}
