use leptoaster::expect_toaster;
use leptos::prelude::*;
use leptos_meta::*;

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

        <div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full mx-auto mt-10">
                <h1 class="text-center text-3xl font-bold text-gray-800 dark:text-gray-100 mb-6">
                    "Profile Settings"
                </h1>

                <Suspense fallback=move || view! { <p class="text-center text-gray-600 dark:text-gray-400">"Loading user settings..."</p> }>
                    <ErrorBoundary fallback=|_| {
                        view! {
                            <p class="text-center text-red-500">
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

                <hr class="my-8 border-gray-300 dark:border-gray-700"/>

                <ActionForm action=logout>
                    <button type="submit" class="w-full bg-red-500 hover:bg-red-600 text-white py-2 px-4 rounded-lg">
                        "Logout"
                    </button>
                </ActionForm>
            </div>
        </div>

        // <div class="settings-page">
        //     <div class="container page">
        //         <div class="row">
        //             <div class="col-md-6 offset-md-3 col-xs-12">
        //                 <h1 class="text-xs-center">"Profile Settings"</h1>
        //
        //                 <Suspense fallback=move || view! { <p>"Loading user settings"</p> }>
        //                     <ErrorBoundary fallback=|_| {
        //                         view! {
        //                             <p>
        //                                 "There was a problem while fetching settings, try again later"
        //                             </p>
        //                         }
        //                     }>
        //                         {move || {
        //                             resource
        //                                 .get()
        //                                 .map(move |x| {
        //                                     x.map(move |user| view! { <SettingsViewForm user /> })
        //                                 })
        //                         }}
        //                     </ErrorBoundary>
        //                 </Suspense>
        //                 <hr />
        //                 <ActionForm action=logout>
        //                     <button type="submit" class="btn btn-outline-danger">
        //                         "Or click here to logout."
        //                     </button>
        //                 </ActionForm>
        //             </div>
        //         </div>
        //     </div>
        // </div>
    }
}

#[component]
pub fn SettingsViewForm(user: User) -> impl IntoView {
    let settings_action: ServerAction<SettingsUpdateAction> = ServerAction::new();
    let result = settings_action.value();

    Effect::new(move |_| {
        if let Some(res) = result.get() {
            let toaster = expect_toaster();
            match res {
                Ok(SettingsUpdateError::Successful) => {
                    toaster.success("Settings updated successfully");
                }
                Ok(SettingsUpdateError::PasswordsNotMatch) => {
                    toaster.error("Passwords do not match");
                }
                Ok(SettingsUpdateError::ValidationError(msg)) => {
                    toaster.error(format!("Validation error: {msg}"));
                }
                Err(e) => {
                    toaster.error(format!("Server error: {e}"));
                }
            }
        }
    });

    view! {
        <ActionForm
            action=settings_action
            class:p-8=true
            class:shadow-md=true
            class:rounded-lg=true
            class:mt-8=true
            class:space-y-6=true
        >
            <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Username
                </label>
                <input
                    disabled
                    type="text"
                    placeholder="Your Username"
                    value=user.username()
                    class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                />
            </div>

            <div>
                <label for="bio" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Bio
                </label>
                <textarea
                    name="bio"
                    rows="5"
                    placeholder="Short bio about you and your work"
                    prop:value=user.bio().unwrap_or_default()
                    class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                ></textarea>
            </div>

            <div>
                <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Email
                </label>
                <input
                    name="email"
                    type="text"
                    placeholder="Email"
                    value=user.email()
                    class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                />
            </div>

            <div>
                <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Password
                </label>
                <input
                    name="password"
                    type="password"
                    placeholder="New Password"
                    class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                />
                <input
                    name="confirm_password"
                    type="password"
                    placeholder="Confirm Password"
                    class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                />
            </div>

            <button
                type="submit"
                class="w-full flex justify-center rounded-md border border-transparent bg-green-400 hover:bg-green-500 px-4 py-2 text-sm font-medium text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-green-300 focus:ring-offset-2"
            >
                "Update Settings"
            </button>
        </ActionForm>

        // <ActionForm action=settings_action>
        //     <fieldset>
        //         <div class="form-group">
        //             <input
        //                 disabled
        //                 class="form-control form-control-lg"
        //                 type="text"
        //                 placeholder="Your Name"
        //                 value=user.username()
        //             />
        //         </div>
        //
        //         <div class="form-group">
        //             <textarea
        //                 name="bio"
        //                 class="form-control form-control-lg"
        //                 rows="8"
        //                 placeholder="Short bio about you"
        //                 prop:value=user.bio().unwrap_or_default()
        //             ></textarea>
        //         </div>
        //
        //         <div class="form-group">
        //             <input
        //                 name="email"
        //                 class="form-control form-control-lg"
        //                 type="text"
        //                 placeholder="Email"
        //                 value=user.email()
        //             />
        //         </div>
        //
        //         <div class="form-group">
        //             <input
        //                 name="password"
        //                 class="form-control form-control-lg"
        //                 type="password"
        //                 placeholder="New Password"
        //             />
        //             <input
        //                 name="confirm_password"
        //                 class="form-control form-control-lg"
        //                 type="password"
        //                 placeholder="Confirm New Password"
        //             />
        //         </div>
        //
        //         <button class="btn btn-lg btn-primary pull-xs-right" type="submit">
        //             "Update Settings"
        //         </button>
        //     </fieldset>
        // </ActionForm>
    }
}
