use leptos::prelude::*;
use leptos_meta::*;
use leptos_toaster::{Toaster, ToasterPosition};

use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SettingsUpdateError {
    PasswordsNotMatch,
    Successful,
    ValidationError(String),
}

#[tracing::instrument]
#[server(SettingsUpdateAction, "/api")]
pub async fn settings_update(
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsUpdateError, ServerFnError> {
    let user = get_user().await?;
    let username = user.username();
    let user = match update_user_validation(user, bio, email, password, &confirm_password) {
        Ok(x) => x,
        Err(x) => return Ok(x),
    };
    user.update()
        .await
        .map(|_| SettingsUpdateError::Successful)
        .map_err(move |x| {
            tracing::error!(
                "Problem while updating user: {} with error {}",
                username,
                x.to_string()
            );
            ServerFnError::ServerError("Problem while updating user".into())
        })
}

fn update_user_validation(
    mut user: User,
    bio: String,
    email: String,
    password: String,
    confirm_password: &str,
) -> Result<User, SettingsUpdateError> {
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsUpdateError::PasswordsNotMatch);
        }
        user = user
            .set_password(password)
            .map_err(SettingsUpdateError::ValidationError)?;
    }

    user.set_email(email)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_bio(bio)
        .map_err(SettingsUpdateError::ValidationError)
}

#[cfg(feature = "ssr")]
async fn get_user() -> Result<User, ServerFnError> {
    let Some(username) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".to_string(),
        ));
    };

    User::get(username).await.map_err(|x| {
        let err = x.to_string();
        tracing::error!("problem while getting the user {err}");
        ServerFnError::ServerError(err)
    })
}

#[tracing::instrument]
#[server(SettingsGetAction, "/api", "GetJson")]
pub async fn settings_get() -> Result<User, ServerFnError> {
    get_user().await
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct UserGet {
    username: String,
    email: String,
    bio: Option<String>,
}

#[component]
pub fn Settings(logout: crate::auth::LogoutSignal) -> impl IntoView {
    let resource = Resource::new(|| (), move |_| settings_get());

    view! {
        <Title text="Settings" />

        <div class="settings-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Profile Settings"</h1>

                        <Suspense fallback=move || view! { <p>"Loading user settings"</p> }>
                            <ErrorBoundary fallback=|_| {
                                view! {
                                    <p>
                                        "There was a problem while fetching settings, try again later"
                                    </p>
                                }
                            }>
                                {move || {
                                    resource
                                        .get()
                                        .map(move |x| {
                                            x.map(move |user| view! { <SettingsViewForm user /> })
                                        })
                                }}
                            </ErrorBoundary>
                        </Suspense>
                        <hr />
                        <ActionForm action=logout>
                            <button type="submit" class="btn btn-outline-danger">
                                "Or click here to logout."
                            </button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SettingsViewForm(user: User) -> impl IntoView {
    let settings_action: ServerAction<SettingsUpdateAction> = ServerAction::new();
    let result = settings_action.value();

    // Use updated Effect API
    Effect::new(move |_| {
        if let Some(res) = result.get() {
            match res {
                Ok(SettingsUpdateError::Successful) => {
                    toast!(info, "✅ Settings updated successfully!");
                }
                Ok(SettingsUpdateError::PasswordsNotMatch) => {
                    toast!(error, "❌ Passwords do not match.");
                }
                Ok(SettingsUpdateError::ValidationError(msg)) => {
                    toast!(error, format!("❌ Validation error: {msg}"));
                }
                Err(e) => {
                    toast!(error, format!("❌ Server error: {e}"));
                }
            }
        }
    });

    view! {
        <>
            // Render toaster here *only* if your layout doesn’t already include it globally
            <Toaster position=ToasterPosition::BottomCenter />

            <ActionForm action=settings_action>
                <fieldset>
                    <div class="form-group">
                        <input
                            disabled
                            class="form-control form-control-lg"
                            type="text"
                            placeholder="Your Name"
                            value=user.username()
                        />
                    </div>

                    <div class="form-group">
                        <textarea
                            name="bio"
                            class="form-control form-control-lg"
                            rows="8"
                            placeholder="Short bio about you"
                            prop:value=user.bio().unwrap_or_default()
                        ></textarea>
                    </div>

                    <div class="form-group">
                        <input
                            name="email"
                            class="form-control form-control-lg"
                            type="text"
                            placeholder="Email"
                            value=user.email()
                        />
                    </div>

                    <div class="form-group">
                        <input
                            name="password"
                            class="form-control form-control-lg"
                            type="password"
                            placeholder="New Password"
                        />
                        <input
                            name="confirm_password"
                            class="form-control form-control-lg"
                            type="password"
                            placeholder="Confirm New Password"
                        />
                    </div>

                    <button class="btn btn-lg btn-primary pull-xs-right" type="submit">
                        "Update Settings"
                    </button>
                </fieldset>
            </ActionForm>
        </>
    }
}
