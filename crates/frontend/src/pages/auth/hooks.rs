use leptos::prelude::*;
use leptos_router::*;

// Custom hook to handle navigation
pub fn use_navigate() -> Box<dyn Fn(&str, NavigateOptions)> {
    // Use the navigate function from leptos_router
    use_navigate()
}
