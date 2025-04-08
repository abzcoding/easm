use crate::api::{ApiClient, ApiError};
use crate::pages::auth::hooks::use_auth_navigate;
use crate::utils::{clear_auth_token, get_auth_token, save_auth_token};
use leptos::prelude::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

mod hooks;

#[derive(Serialize, Debug)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct AuthResponse {
    token: String,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let navigate = use_auth_navigate();
    let api_client = ApiClient::new("http://localhost:3000".to_string());

    // Check if we already have a token
    let api_client_clone = api_client.clone();
    let check_existing_token = move || {
        // If we already have a token, redirect to dashboard
        if let Some(token) = get_auth_token() {
            // Set token in API client
            let mut client = api_client_clone.clone();
            client.set_token(token);

            // Navigate to dashboard
            navigate("/dashboard", NavigateOptions::default());
        }
    };

    // Run the check on mount
    let _effect = Effect::new(move |_| {
        check_existing_token();
    });

    // Create a stable function that can be called multiple times (FnMut)
    let handle_submit = move |_| {
        let email_val = email.get();
        let password_val = password.get();
        let client = api_client.clone();
        let set_error = set_error;
        let set_loading = set_loading;

        // Basic validation
        if email_val.is_empty() || password_val.is_empty() {
            set_error("Email and password are required".to_string());
            return;
        }

        // Clear any previous errors
        set_error(String::new());
        set_loading(true);

        // Create login request
        let login_request = LoginRequest {
            email: email_val,
            password: password_val,
        };

        // Use spawn_local for WASM-safe async handling
        spawn_local(async move {
            // Make API call to login
            match client
                .post::<AuthResponse, _>("/api/auth/login", &login_request)
                .await
            {
                Ok(response) => {
                    // Save token
                    match save_auth_token(&response.token) {
                        Ok(_) => {
                            // Set token in API client
                            let mut client_with_token = client.clone();
                            client_with_token.set_token(response.token);

                            // Navigate to dashboard after login
                            // Use window.location.href instead of navigate since we're in an async context
                            if let Some(window) = web_sys::window() {
                                if let Ok(location) = window.location().href() {
                                    let base =
                                        location.split('/').take(3).collect::<Vec<_>>().join("/");
                                    let dashboard_url = format!("{}/dashboard", base);
                                    let _ = window.location().set_href(&dashboard_url);
                                }
                            }
                            set_loading(false);
                        }
                        Err(e) => {
                            set_error(format!("Failed to save token: {}", e));
                            set_loading(false);
                        }
                    }
                }
                Err(e) => {
                    // Handle different API errors
                    let error_message = match e {
                        ApiError::AuthError(msg) => format!("Authentication error: {}", msg),
                        ApiError::BadRequest(msg) => format!("Bad request: {}", msg),
                        ApiError::NotFound => "Login service not available".to_string(),
                        ApiError::ServerError(msg) => format!("Server error: {}", msg),
                        ApiError::NetworkError(msg) => format!("Network error: {}", msg),
                        ApiError::DeserializationError(msg) => {
                            format!("Could not process response: {}", msg)
                        }
                    };
                    set_error(error_message);
                    set_loading(false);
                }
            }
        });
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        handle_submit(());
    };

    view! {
        <div style="display: flex; align-items: center; justify-content: center; min-height: 100vh; background-color: #f0f2f5;">
            <div style="padding: 2rem; border-radius: 8px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1); background-color: white; max-width: 400px; width: 90%;">
                <h1 style="text-align: center; margin-bottom: 1.5rem; color: #333; font-size: 1.8rem; font-weight: 600;">"Login to EASM"</h1>

                <div class="alert alert-danger" style=move || format!("padding: 0.75rem; margin-bottom: 1rem; border: 1px solid transparent; border-radius: 4px; color: #721c24; background-color: #f8d7da; border-color: #f5c6cb; {}", if error.get().is_empty() { "display: none;" } else { "display: block;" })>
                    {move || error.get()}
                </div>

                <form on:submit=on_submit>
                    <div style="margin-bottom: 1rem;">
                        <label for="email" style="display: block; margin-bottom: 0.5rem; font-weight: 500; color: #555;">"Email"</label>
                        <input
                            type="email"
                            id="email"
                            name="email"
                            style="width: 100%; padding: 0.75rem; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box;"
                            placeholder="Enter your email"
                            on:input=move |ev| {
                                let input = event_target::<HtmlInputElement>(&ev);
                                set_email(input.value());
                            }
                        />
                    </div>

                    <div style="margin-bottom: 1.5rem;">
                        <label for="password" style="display: block; margin-bottom: 0.5rem; font-weight: 500; color: #555;">"Password"</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            style="width: 100%; padding: 0.75rem; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box;"
                            placeholder="Enter your password"
                            on:input=move |ev| {
                                let input = event_target::<HtmlInputElement>(&ev);
                                set_password(input.value());
                            }
                        />
                    </div>

                    <div style="margin-bottom: 0.75rem;">
                        <button type="submit"
                            style=move || format!("width: 100%; padding: 0.85rem; border: none; border-radius: 4px; background-color: #007bff; color: white; font-size: 1rem; cursor: {}; transition: background-color 0.2s ease; {}",
                                if loading.get() { "not-allowed" } else { "pointer" },
                                if loading.get() { "opacity: 0.7;" } else { "" }
                            )
                            disabled=loading
                        >
                            {move || if loading.get() { "Logging in..." } else { "Login" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
