use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::*;

use crate::components::QueryMeta;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct QueryResult {
    pub(super) query: crate::models::Query,
    pub(super) logged_user: Option<crate::models::User>,
}

#[server(GetQueryAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_query(query_id: i64) -> Result<QueryResult, ServerFnError> {
    Ok(QueryResult {
        query: crate::models::Query::for_query(query_id)
            .await
            .map_err(|x| {
                let err = format!("Error while getting user_profile queries: {x:?}");
                tracing::error!("{err}");
                ServerFnError::new("Could not retrieve queries, try again later")
            })?,
        logged_user: crate::auth::current_user().await.ok(),
    })
}

#[tracing::instrument]
#[component]
pub fn Query(user: crate::auth::UserSignal) -> impl IntoView {
    let params = use_params_map();
    let query = Resource::new(
        move || params.get().get("query_id").cloned().unwrap_or_default(),
        |query_id| async { get_query(query_id).await },
    );

    let title = RwSignal::new(String::from("Loading"));

    view! {
        <Title text=move || title.get()/>

        <Suspense fallback=move || view! { <p>"Loading Query"</p> }>
            <ErrorBoundary fallback=|_| {
                view! { <p class="error-messages text-xs-center">"Something went wrong, please try again later."</p>}
            }>
                {move || {
                    query.get().map(move |x| {
                        x.map(move |query_result| {
                            title.set(query_result.query.query_id.to_string());
                            view! {
                                <QueryPage user result=query_result />
                            }
                        })
                    })
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn QueryPage(user: crate::auth::UserSignal, result: QueryResult) -> impl IntoView {
    let query_signal = RwSignal::new(result.query.clone());
    let user_signal = RwSignal::new(result.logged_user);
    let tag_list = result.query.tag_list;

    view! {
        <div class="query-page">
            <div class="banner">
                <div class="container">
                    <h1>{result.query.title}</h1>
                    <QueryMeta user query=query_signal is_preview=false />
                </div>
            </div>

            <div class="container page">
                <div class="row query-content">
                    <div class="col-md-12">
                        <p>{result.query.body}</p>
                    </div>
                </div>

                <ul class="tag-list">
                    <For
                        each=move || tag_list.clone().into_iter().enumerate()
                        key=|(i, _)| *i
                        children=|(_, a)| {view!{<li class="tag-default tag-pill tag-outline">{a}</li>}}
                    />
                </ul>

                <hr />

                <div class="query-actions">
                    <div class="row" style="justify-content: center;">
                        <QueryMeta username query=query_signal is_preview=false />
                    </div>
                </div>

                <div class="row">
                    <CommentSection username query=query_signal user=user_signal />
                </div>
            </div>
        </div>
    }
}

#[server(PostCommentAction, "/api")]
#[tracing::instrument]
pub async fn post_comment(slug: String, body: String) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::insert(slug, logged_user, body)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[server(GetCommentsAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_comments(slug: String) -> Result<Vec<crate::models::Comment>, ServerFnError> {
    crate::models::Comment::get_all(slug).await.map_err(|x| {
        let err = format!("Error while posting a comment: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not post a comment, try again later".into())
    })
}

#[server(DeleteCommentsAction, "/api")]
#[tracing::instrument]
pub async fn delete_comment(id: i32) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::delete(id, logged_user)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[component]
fn CommentSection(
    logged_user: crate::auth::UserSignal,
    query: crate::components::QuerySignal,
    user: RwSignal<Option<crate::models::User>>,
) -> impl IntoView {
    let comments_action: ServerAction<PostCommentAction> = ServerAction::new();
    let result = comments_action.version();
    let reset_comment = RwSignal::new("");
    let comments = Resource::new(
        move || (result.get(), query.with(|a| a.slug.to_string())),
        move |(_, a)| async move {
            reset_comment.set("");
            get_comments(a).await
        },
    );

    view! {
        <div class="col-xs-12 col-md-8 offset-md-2">
            <Show when=move || username.with(Option::is_some) fallback=|| ()>
                <ActionForm action=comments_action class="card comment-form">
                    <input name="slug" type="hidden" value=move || query.with(|x| x.slug.to_string()) />
                    <div class="card-block">
                        <textarea name="body" prop:value=move || reset_comment.get() class="form-control" placeholder="Write a comment..." rows="3"></textarea>
                    </div>
                    <div class="card-footer">
                        <img src=move || user.with(|x| x.as_ref().map(crate::models::User::image).unwrap_or_default()) class="comment-author-img" />
                        <button class="btn btn-sm btn-primary" type="submit">
                            "Post Comment"
                        </button>
                    </div>
                </ActionForm>
            </Show>
            <Suspense fallback=move || view! {<p>"Loading Comments from the query"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
                }>
                    {move || comments.get().map(move |x| x.map(move |c| {
                        view! {
                            <For each=move || c.clone().into_iter().enumerate()
                                key=|(i, _)| *i
                                children=move |(_, comment)| {
                                    let comment = RwSignal::new(comment);
                                    view!{<Comment username comment comments />}
                                }/>
                        }
                    }))}
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}

#[component]
fn Comment<T: 'static + Clone, S: 'static>(
    username: crate::auth::UsernameSignal,
    comment: RwSignal<crate::models::Comment>,
    comments: Resource<T, S>,
) -> impl IntoView {
    let user_link = move || format!("/profile/{}", comment.with(|x| x.username.to_string()));
    let user_image = move || comment.with(|x| x.user_image.clone().unwrap_or_default());
    let delete_c = create_server_action::<DeleteCommentsAction>();
    let delete_result = delete_c.value();

    create_effect(move |_| {
        if let Some(Ok(())) = delete_result.get() {
            tracing::info!("comment deleted!");
            comments.refetch();
        }
    });

    view! {
        <div class="card">
            <div class="card-block">
                <p class="card-text">{move || comment.with(|x| x.body.to_string())}</p>
            </div>
            <div class="card-footer">
                <A href=user_link class="comment-author">
                    <img src=user_image class="comment-author-img" />
                </A>
                " "
                <A href=user_link class="comment-author">{move || comment.with(|x| x.username.to_string())}</A>
                <span class="date-posted">{move || comment.with(|x| x.created_at.to_string())}</span>
                <Show
                    when=move || {username.get().unwrap_or_default() == comment.with(|x| x.username.to_string())}
                    fallback=|| ()>
                    <ActionForm action=delete_c class="comment-author">
                        <input type="hidden" name="id" value=move || comment.with(|x| x.id) />
                        <button class="btn btn-sm" type="submit"><i class="ion-trash-b"></i></button>
                    </ActionForm>
                </Show>
            </div>
        </div>
    }
}
