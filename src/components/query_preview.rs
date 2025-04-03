use leptos::prelude::*;
use leptos_router::components::*;

pub type QuerySignal = RwSignal<crate::models::Query>;

type QueriesType = Resource<Result<Vec<crate::models::Query>, ServerFnError>>;

#[component]
pub fn QueryPreviewList(user: crate::auth::UserSignal, queries: QueriesType) -> impl IntoView {
    let queries_view = move || {
        queries.with(move |x| {
            x.clone().map(move |res| {
                view! {
                    <For
                        each=move || res.clone().unwrap_or_default().into_iter().enumerate()
                        key=|(i, _)| *i
                        children=move |(_, query): (usize, crate::models::Query)| {
                            let query = RwSignal::new(query);
                            view! {
                                <QueryPreview query=query user=user />
                            }
                        }
                    />
                }
            })
        })
    };

    view! {
        <Suspense fallback=move || view! {<p>"Loading Queries"</p> }>
            <ErrorBoundary fallback=|_| {
                view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
            }>
                {queries_view}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn QueryPreview(user: crate::auth::UserSignal, query: QuerySignal) -> impl IntoView {
    view! {
        <div class="query-preview">
            <QueryMeta user=user query=query is_preview=true />
            <A href=move || format!("/query/{}", query.with(|x| x.query_id.clone())) class:preview-link=true>
                <h1>{move || format!("{} as of {}", query.with(|x| x.status.clone()), query.with(|x| x.status_updated_at.clone()))}</h1>
                <p>{move || query.with(|x| x.description.clone().unwrap_or_default())}</p>
                <span class="btn">"See more..."</span>
            </A>
        </div>
    }
}

#[component]
pub fn QueryMeta(
    user: crate::auth::UserSignal,
    query: QuerySignal,
    is_preview: bool,
) -> impl IntoView {
    let editor_ref = move || format!("/editor/{}", query.with(|x| x.query_id.clone()));
    let profile_ref = move || {
        format!(
            "/profile/{}",
            user.with(|x| x.clone().unwrap_or_default().1)
        )
    };

    let delete_a: ServerAction<DeleteQueryAction> = ServerAction::new();

    view! {
        <div class="query-meta">
            <A href=profile_ref>"Query"</A>
            <div class="info">
                <A href=profile_ref class:author=true>{move || query.with(|x| x.user_id.to_string())}</A>
                <span class="date">{move || query.with(|x| x.created_at.to_string())}</span>
            </div>
            <Show
                when=move || is_preview
                fallback=move || {
                    view! {
                        <Show
                            when=move || {user.get().unwrap_or_default().0 == query.with(|x| x.user_id)}
                            fallback=move || {}
                        >
                            <A class:btn=true class:btn-sm=true class:btn-outline-secondary=true href=editor_ref>
                                <i class="ion-compose"></i>" Edit query"
                            </A>
                            <ActionForm action=delete_a class:inline=true>
                                <input type="hidden" name="slug" value=move || query.with(|x| x.query_id.clone()) />
                                <button type="submit" class="btn btn-sm btn-outline-secondary">
                                    <i class="ion-trash-a"></i>" Delete query"
                                </button>
                            </ActionForm>
                        </Show>
                    }
                }
            >
                <div>"Button Fav standin"</div>
            </Show>
        </div>
    }
}

#[server(DeleteQueryAction, "/api")]
#[tracing::instrument]
pub async fn delete_query(query_id: i64) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };
    let redirect_profile = format!("/profile/{}", logged_user);

    crate::models::Query::delete(query_id)
        .await
        .map(move |_| {
            leptos_axum::redirect(&redirect_profile);
        })
        .map_err(|x| {
            let err = format!("Error while deleting an query: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not delete the query, try again later".into())
        })
}
