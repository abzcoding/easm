use crate::pages::auth::hooks::use_navigate;
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;
use web_sys::HtmlInputElement;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(String::new());
    let navigate = use_navigate();

    // Create a stable function that can be called multiple times (FnMut)
    let handle_submit = create_action(move |_: &()| {
        let username_val = username.get();
        let password_val = password.get();
        let nav = navigate.clone();

        async move {
            // Basic validation
            if username_val.is_empty() || password_val.is_empty() {
                set_error("Username and password are required".to_string());
                return false;
            }

            // Clear any previous errors
            set_error(String::new());

            // TODO: In a real app, make API call to login
            // For now, just simulate successful login

            // Navigate to dashboard after "login"
            nav("/", NavigateOptions::default());
            true
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        handle_submit.dispatch(());
    };

    view! {
        <div class="login-container">
            <div class="card login-card">
                <h1 class="login-title">"Login to EASM"</h1>

                {move || {
                    let err = error.get();
                    if !err.is_empty() {
                        view! {
                            <div class="alert alert-danger">
                                {err}
                            </div>
                        }
                    } else {
                        view! { <div></div> }
                    }
                }}

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label for="username" class="form-label">"Username"</label>
                        <input
                            type="text"
                            id="username"
                            name="username"
                            class="form-input"
                            placeholder="Enter your username"
                            on:input=move |ev| {
                                let input = event_target::<HtmlInputElement>(&ev);
                                set_username(input.value());
                            }
                        />
                    </div>

                    <div class="form-group">
                        <label for="password" class="form-label">"Password"</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            class="form-input"
                            placeholder="Enter your password"
                            on:input=move |ev| {
                                let input = event_target::<HtmlInputElement>(&ev);
                                set_password(input.value());
                            }
                        />
                    </div>

                    <div class="form-group text-center">
                        <button type="submit" class="btn btn-primary login-btn">
                            "Login"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
