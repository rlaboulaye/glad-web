use leptos::prelude::*;

use crate::models::User;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[tracing::instrument]
pub fn validate_signup(
    username: String,
    bio: String,
    email: String,
    password: String,
) -> Result<User, String> {
    User::default()
        .set_username(username)?
        .set_bio(bio)?
        .set_password(password)?
        .set_email(email)
}

#[tracing::instrument]
#[server(SignupAction, "/api")]
pub async fn signup_action(
    username: String,
    bio: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    match validate_signup(username.clone(), bio, email, password) {
        Ok(user) => match user.insert().await {
            Ok(_) => {
                crate::auth::set_username(username).await;
                leptos_axum::redirect("/");
                Ok(SignupResponse::Success)
            }
            Err(x) => {
                let x = x.to_string();
                Ok(if x.contains("user_email_key") {
                    SignupResponse::CreateUserError("Duplicated email".to_string())
                } else if x.contains("user_pkey") {
                    SignupResponse::CreateUserError("Duplicated user".to_string())
                } else {
                    tracing::error!("error from DB: {}", x);
                    SignupResponse::CreateUserError(
                        "There is an unknown problem, try again later".to_string(),
                    )
                })
            }
        },
        Err(x) => Ok(SignupResponse::ValidationError(x)),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum LoginMessages {
    Successful,
    Unsuccessful,
}

#[server(LoginAction, "/api")]
#[tracing::instrument]
pub async fn login_action(
    username: String,
    password: String,
) -> Result<LoginMessages, ServerFnError> {
    let hash = sqlx::query_scalar!("SELECT password FROM user where username=$1", username,)
        .fetch_one(crate::database::get_db())
        .await
        .unwrap_or_default();
    match crate::models::verify_password(password, hash).await {
        Ok(_) => {
            crate::auth::set_username(username).await;
            leptos_axum::redirect("/");
            Ok(LoginMessages::Successful)
        }
        Err(_) => {
            let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();
            response_options.set_status(axum::http::StatusCode::FORBIDDEN);
            Ok(LoginMessages::Unsuccessful)
        }
    }
}

#[server(LogoutAction, "/api")]
#[tracing::instrument]
pub async fn logout_action() -> Result<(), ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(crate::auth::REMOVE_COOKIE)
            .expect("header value couldn't be set"),
    );
    leptos_axum::redirect("/login");
    Ok(())
}

#[server(CurrentUserAction, "/api")]
#[tracing::instrument]
pub async fn current_user() -> Result<Option<User>, ServerFnError> {
    let Some(logged_user) = super::get_username() else {
        return Ok(None);
    };
    User::get(logged_user).await.map(Some).map_err(|err| {
        tracing::error!("problem while retrieving current_user: {err:?}");
        ServerFnError::Request("failed to load user from database".into())
    })
}
