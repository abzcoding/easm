use leptos::prelude::*;
use leptos_router::*;
use web_sys::{window, Location};

// Custom hook to handle navigation
pub fn use_auth_navigate() -> Box<dyn Fn(&str, NavigateOptions)> {
    Box::new(move |path: &str, options: NavigateOptions| {
        // Get window and location
        if let Some(window) = window() {
            if let Ok(location) = window.location().set_href(path) {
                // No need to do anything with the result
            }
        }
    })
}
