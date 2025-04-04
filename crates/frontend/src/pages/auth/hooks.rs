use leptos::prelude::*;
use leptos_router::{NavigateOptions, use_navigate as leptos_use_navigate};

// Re-export the navigate function from leptos_router
pub fn use_navigate() -> impl Fn(&str, NavigateOptions) {
    leptos_use_navigate()
} 