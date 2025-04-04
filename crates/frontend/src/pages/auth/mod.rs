use crate::api::ApiClient;
use crate::pages::auth::hooks::use_navigate;
use crate::utils::{clear_auth_token, get_auth_token, save_auth_token};
use leptos::prelude::*;
use leptos_router::*;
use web_sys::HtmlInputElement;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());
    let navigate = use_navigate();
    let api_client = ApiClient::new("http://localhost:8080/api".to_string());

    // Clone values for the first closure
    let navigate_clone1 = navigate.clone();
    let api_client_clone1 = api_client.clone();

    // Check if we already have a token
    let check_existing_token = move || {
        // If we already have a token, redirect to dashboard
        if let Some(token) = get_auth_token() {
            // Set token in API client
            let mut client = api_client_clone1.clone();
            client.set_token(token);

            // Navigate to dashboard
            navigate_clone1("/", NavigateOptions::default());
        }
    };

    // Run the check on mount
    let _effect = Effect::new(move |_| {
        check_existing_token();
    });

    // Clone values for the second closure
    let navigate_clone2 = navigate.clone();
    let api_client_clone2 = api_client.clone();

    // Create a stable function that can be called multiple times (FnMut)
    let handle_submit = Action::new(move |_: &()| {
        let username_val = username.get();
        let password_val = password.get();
        let nav = navigate_clone2.clone();
        let mut client = api_client_clone2.clone();

        async move {
            // Basic validation
            if username_val.is_empty() || password_val.is_empty() {
                set_error("Username and password are required".to_string());
                return false;
            }

            // Clear any previous errors
            set_error(String::new());

            // TODO: In a real app, make API call to login
            // For now, just simulate successful login with a fake token
            let fake_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiIxMjMiLCJ1c2VybmFtZSI6InRlc3R1c2VyIn0.aBcDeFgHiJkLmNoPqRsTuVwXyZ";

            // Save token
            match save_auth_token(fake_token) {
                Ok(_) => {
                    // Set token in API client
                    client.set_token(fake_token.to_string());

                    // Navigate to dashboard after login
                    nav("/", NavigateOptions::default());
                    true
                }
                Err(e) => {
                    set_error(format!("Failed to save token: {}", e));
                    false
                }
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        handle_submit.dispatch(());
    };

    let logout = move |_| {
        // Clear auth token
        if let Err(e) = clear_auth_token() {
            set_error(format!("Failed to logout: {}", e));
        }
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
                        <label for="username" style="display: block; margin-bottom: 0.5rem; font-weight: 500; color: #555;">"Username"</label>
                        <input
                            type="text"
                            id="username"
                            name="username"
                            style="width: 100%; padding: 0.75rem; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box;"
                            placeholder="Enter your username"
                            on:input=move |ev| {
                                let input = event_target::<HtmlInputElement>(&ev);
                                set_username(input.value());
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
                        <button type="submit" style="width: 100%; padding: 0.85rem; border: none; border-radius: 4px; background-color: #007bff; color: white; font-size: 1rem; cursor: pointer; transition: background-color 0.2s ease;">
                            "Login"
                        </button>
                    </div>

                    <div>
                        <button type="button" style="width: 100%; padding: 0.85rem; border: 1px solid #6c757d; border-radius: 4px; background-color: transparent; color: #6c757d; font-size: 1rem; cursor: pointer; transition: background-color 0.2s ease, color 0.2s ease;" on:click=logout>
                            "Logout"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
