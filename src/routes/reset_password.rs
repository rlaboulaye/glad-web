use std::env;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{hooks::use_query, params::Params};

#[cfg(feature = "ssr")]
struct EmailCredentials {
    email: String,
    passwd: String,
    smtp_server: String,
}

#[cfg(feature = "ssr")]
static EMAIL_CREDS: std::sync::OnceLock<EmailCredentials> = std::sync::OnceLock::new();

#[tracing::instrument]
#[server(ResetPasswordAction1, "/api")]
pub async fn reset_password_1(email: String) -> Result<String, ServerFnError> {
    if let Err(x) = crate::models::User::get_email(email.clone()).await {
        let err = format!("Bad email : {x:?}");
        tracing::error!("{err}");
    } else {
        let creds = EMAIL_CREDS.get_or_init(|| EmailCredentials {
            email: env::var("MAILER_EMAIL").unwrap(),
            passwd: env::var("MAILER_PASSWD").unwrap(),
            smtp_server: env::var("MAILER_SMTP_SERVER").unwrap(),
        });
        let host = leptos_axum::extract::<axum::extract::Host>().await?.0;
        let schema = if cfg!(debug_assertions) {
            "http"
        } else {
            "https"
        };
        let token = crate::auth::encode_token(crate::auth::TokenClaims {
            sub: email.clone(),
            exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3_600,
        })
        .unwrap();
        let uri = format!("{}://{}/reset_password?token={}", schema, host, token);
        // Build a simple multipart message
        let message = mail_send::mail_builder::MessageBuilder::new()
            .from(("Realworld Leptos", creds.email.as_str()))
            .to(vec![("GLADdb", email.as_str())])
            .subject("Your password reset from GLADdb")
            .text_body(format!(
                "You can reset your password accessing the following link: {uri}"
            ));

        // Connect to the SMTP submissions port, upgrade to TLS and
        // authenticate using the provided credentials.
        mail_send::SmtpClientBuilder::new(creds.smtp_server.as_str(), 587)
            .implicit_tls(false)
            .credentials((creds.email.as_str(), creds.passwd.as_str()))
            .connect()
            .await
            .unwrap()
            .send(message)
            .await
            .unwrap();
    }
    return Ok(String::from("Check your email"));
}

fn validate_reset(password: String, confirm: String) -> bool {
    password == confirm
}

#[tracing::instrument]
#[server(ResetPasswordAction2, "/api")]
pub async fn reset_password_2(
    token: String,
    password: String,
    confirm: String,
) -> Result<String, ServerFnError> {
    let mut message = String::from("Something went wrong, try again later");
    if !validate_reset(password.clone(), confirm) {
        return Ok(message);
    }
    let Ok(claims) = crate::auth::decode_token(token.as_str()) else {
        tracing::info!("Invalid token provided");
        return Ok(message);
    };
    let email = claims.claims.sub;
    let Ok(user) = crate::models::User::get_email(email.clone()).await else {
        tracing::info!("User does not exist");
        return Ok(message);
    };
    match user.set_password(password) {
        Ok(u) => {
            if let Err(error) = u.update().await {
                tracing::error!(email, ?error, "error while resetting the password");
            } else {
                // A real password reset would have a list of issued tokens and invalidation over
                // the used ones. As this would grow much bigger in complexity, I prefer to write
                // down this security vulnerability and left it simple :)
                message = String::from("Password successfully reset, please, proceed to login");
            }
        }
        Err(x) => {
            message = x;
        }
    }
    Ok(message)
}

#[derive(Params, PartialEq)]
struct TokenQuery {
    token: Option<String>,
}

#[component]
pub fn ResetPassword() -> impl IntoView {
    let q = use_query::<TokenQuery>();
    view! {
        <Title text="Reset Password" />
        <div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-md mt-20">
                <h2 class="text-center text-3xl font-bold tracking-tight text-gray-800 dark:text-gray-100">
                    Reset Password
                </h2>

                {q.with(|x| {
                    if let Ok(token_query) = x {
                        if let Some(token) = token_query.token.as_ref() {
                            // Show ConfirmPassword if token is available
                            return view! {
                                <ConfirmPassword
                                    token=token.to_string()
                                    class:p-8=true
                                    class:shadow-md=true
                                    class:rounded-lg=true
                                    class:space-y-6=true
                                />
                            }
                            .into_any();
                        }
                    }
                    // Otherwise, show AskForEmail form
                    view! {
                        <AskForEmail
                            class:p-8=true
                            class:shadow-md=true
                            class:rounded-lg=true
                            class:space-y-6=true
                        />
                    }
                    .into_any()
                })}
            </div>
        </div>
    }
}

#[component]
fn AskForEmail() -> impl IntoView {
    let reset: ServerAction<ResetPasswordAction1> = ServerAction::new();
    let result_of_call = reset.value();

    let error = move || {
        result_of_call.with(|msg| {
            msg.as_ref()
                .map(|inner| match inner {
                    Ok(x) => x.to_string(),
                    Err(x) => {
                        tracing::error!("Problem while sending email: {x:?}");
                        String::from("There was a problem, try again later")
                    }
                })
                .unwrap_or_default()
        })
    };
    view! {
        <div class="sm:mx-auto sm:w-full sm:max-w-md mt-20">
            <h1 class="text-center text-3xl font-bold tracking-tight text-gray-800 dark:text-gray-100">
                Reset Password
            </h1>

            <p class="mt-2 text-center text-sm text-red-500">
                {error}
            </p>

            <ActionForm
                action=reset
                class:p-8=true
                class:shadow-md=true
                class:rounded-lg=true
                class:space-y-6=true
            >
                <div>
                    <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Email
                    </label>
                    <input
                        name="email"
                        type="email"
                        placeholder="Your Email"
                        class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                    />
                </div>

                <div>
                    <button
                        type="submit"
                        class="w-full flex justify-center rounded-md border border-transparent bg-green-400 hover:bg-green-500 px-4 py-2 text-sm font-medium text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-green-300 focus:ring-offset-2"
                    >
                        Reset Password
                    </button>
                </div>
            </ActionForm>
        </div>
    }
}

#[component]
fn ConfirmPassword(token: String) -> impl IntoView {
    //let reset = create_server_action::<ResetPasswordAction2>();
    let reset: ServerAction<ResetPasswordAction2> = ServerAction::new();
    let result_of_call = reset.value();

    let error = move || {
        result_of_call.with(|msg| {
            msg.as_ref()
                .map(|inner| match inner {
                    Ok(x) => x.to_string(),
                    Err(x) => {
                        tracing::error!("Problem during reset: {x:?}");
                        String::from("There was a problem, try again later")
                    }
                })
                .unwrap_or_default()
        })
    };
    view! {
        <div class="col-md-6 offset-md-3 col-xs-12">
            <h1 class="text-xs-center">"Reset password"</h1>

            <p class="text-xs-center">{error}</p>

            <ActionForm
                action=reset
                on:submit=move |ev| {
                    let Ok(data) = ResetPasswordAction2::from_event(&ev) else {
                        return ev.prevent_default();
                    };
                    if !validate_reset(data.password, data.confirm) {
                        result_of_call.set(Some(Ok(String::from("Password is not the same"))));
                        ev.prevent_default();
                    }
                }
            >
                <fieldset class="form-group">
                    <input
                        name="password"
                        class="form-control form-control-lg"
                        type="password"
                        placeholder="Your new password"
                    />

                    <input
                        name="confirm"
                        class="form-control form-control-lg"
                        type="password"
                        placeholder="Confirm your password"
                    />

                    <input name="token" type="hidden" value=token />
                </fieldset>
                <button class="btn btn-lg btn-primary pull-xs-right">"Reset Password"</button>
            </ActionForm>
        </div>
    }
}
