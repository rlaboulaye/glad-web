use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::*;

use crate::components::QueryPreviewList;

#[server(UserQueriesAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn profile_queries(
    username: String,
    favourites: Option<bool>,
) -> Result<Vec<crate::models::Query>, ServerFnError> {
    crate::models::Query::for_user_profile().await.map_err(|x| {
        let err = format!("Error while getting user_profile queries: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not retrieve queries, try again later".into())
    })
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserProfileModel {
    user: crate::models::User,
    following: Option<bool>,
}

#[server(UserProfileAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn user_profile(username: String) -> Result<UserProfileModel, ServerFnError> {
    let user = crate::models::User::get(username.clone())
        .await
        .map_err(|x| {
            let err = format!("Error while getting user in user_profile: {x:?}");
            tracing::error!("{err}");
            ServerFnError::new("Could not retrieve queries, try again later")
        })?;
    match crate::auth::get_username() {
        Some(lu) => sqlx::query!(
            "SELECT EXISTS(SELECT * FROM Follows WHERE follower=$2 and influencer=$1)",
            username,
            lu,
        )
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|x| {
            let err = format!("Error while getting user in user_profile: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not retrieve queries, try again later".into())
        })
        .map(|x| UserProfileModel {
            user,
            following: x.exists,
        }),
        None => Ok(UserProfileModel {
            user,
            following: None,
        }),
    }
}

#[allow(clippy::redundant_closure)]
#[tracing::instrument]
#[component]
pub fn Profile(user: crate::auth::UserSignal) -> impl IntoView {
    let params = use_params_map();
    let route_user = move || params.with(|x| x.get("user").cloned().unwrap_or_default());
    let query = use_query_map();
    let favourite = move || query.with(|x| x.get("favourites").map(|_| true));

    let user_query_href = move || format!("/profile/{}", route_user());
    let favourites_href = move || format!("{}?favourites=true", user_query_href());

    let queries = Resource::new(
        move || (favourite(), route_user()),
        move |(fav, user)| async move { profile_queries(user, fav).await },
    );

    view! {
        <Title text=move || format!("{}'s profile", route_user()) />
        <div class="profile-page">
            <UserInfo logged_user=user />

            <div class="container">
                <div class="row">
                    <div class="col-xs-12 col-md-10 offset-md-1">
                        <div class="queries-toggle">
                            <ul class="nav nav-pills outline-active">
                                <li class="nav-item">
                                    <a class="nav-link"
                                        class:active=move || !favourite().unwrap_or_default() href=user_query_href>
                                            {move || route_user()}"'s Queries"
                                        </a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link"
                                        class:active=move || favourite().unwrap_or_default()
                                        href=favourites_href>"Favorited Queries"</a>
                                </li>
                            </ul>
                        </div>

                        <QueryPreviewList user=user queries=queries />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn UserInfo(logged_user: crate::auth::UserSignal) -> impl IntoView {
    let params = use_params_map();
    let resource = Resource::new(
        move || (params.with(|x| x.get("user").cloned().unwrap_or_default())),
        move |user| async move { user_profile(user).await },
    );

    view! {
        <div class="user-info">
            <div class="container">
                <div class="row">
                    <div class="col-xs-12 col-md-10 offset-md-1">
                    <Suspense
                        fallback=move || view!{<p>"Loading user profile"</p>}
                    >
                        <ErrorBoundary
                            fallback=|_| {
                                view!{<p>"There was a problem while fetching the user profile, try again later"</p>}
                            }
                        >
                            {move || {
                                resource.get().map(move |x| {
                                    x.map(move |u| {
                                        let username = u.user.username();
                                        let bio = u.user.bio();
                                        let (author, _) = signal(username.to_string());

                                        view!{
                                            <h4>{username}</h4>
                                            <p>{bio.unwrap_or("No bio available".into())}</p>
                                        }
                                    })
                                })
                            }}
                        </ErrorBoundary>
                    </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}
