use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::*;

use crate::models::Query;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    Successful(String),
}

#[cfg_attr(feature = "hydrate", allow(dead_code))]
#[derive(Debug)]
struct QueryUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: std::collections::HashSet<String>,
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn update_query(query_id: i64, description: String) -> Result<String, sqlx::Error> {
    sqlx::query!(
        "UPDATE query SET description=$2 WHERE query_id=$1",
        query_id,
        description,
    )
    .execute(crate::database::get_db())
    .await;
    Ok(query_id.to_string())
}

#[server(EditorAction, "/api")]
#[tracing::instrument]
pub async fn editor_action(
    title: String,
    description: String,
    body: String,
    tag_list: String,
    slug: String,
) -> Result<EditorResponse, ServerFnError> {
    let Some(author) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Ok(EditorResponse::ValidationError(
            "you should be authenticated".to_string(),
        ));
    };
    match update_query(query_id, description).await {
        Ok(x) => {
            leptos_axum::redirect(&format!("/query/{x}"));
            Ok(EditorResponse::Successful(x))
        }
        Err(x) => {
            tracing::error!("EDITOR ERROR: {}", x.to_string());
            Ok(EditorResponse::UpdateError)
        }
    }
}

#[tracing::instrument]
#[component]
pub fn Editor() -> impl IntoView {
    let editor_server_action: ServerAction<EditorAction> = ServerAction::new();
    let result = editor_server_action.value();
    let error = move || {
        result.with(|x| {
            x.as_ref().map_or(true, |y| {
                y.is_err() || !matches!(y, Ok(EditorResponse::Successful(_)))
            })
        })
    };

    let params = use_params_map();
    let query_res = Resource::new(
        move || params.get(),
        |slug| async move {
            if let Some(s) = slug.get("slug") {
                super::get_query(s.to_string()).await
            } else {
                Ok(Query::default())
            }
        },
    );

    view! {
        <Title text="Editor"/>
        <div class="editor-page">
            <div class="container page">
                <div class="row">
                    <p class="text-xs-center"
                        class:text-success=move || !error()
                        class:error-messages=error
                    >
                        <strong>
                            {move || result.with(|x| {
                                let Some(x) = x else {
                                    return String::new();
                                };
                                match x {
                                    Ok(EditorResponse::ValidationError(x)) => {
                                        format!("Problem while validating: {x}")
                                    }
                                    Ok(EditorResponse::UpdateError) => {
                                        "Error while updating the query, please, try again later".into()
                                    }
                                    Ok(EditorResponse::Successful(_)) => {
                                        String::new()
                                    }
                                    Err(x) => format!("Unexpected error: {x}"),
                                }
                            })}
                        </strong>
                    </p>

                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=editor_server_action>
                        <Suspense fallback=move || view! {<p>"Loading Tags"</p> }>
                            <ErrorBoundary fallback=|_| {
                                view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
                            }>
                                {move || query_res.get().map(move |x| x.map(move |a| {
                                    view! {
                                        <fieldset>
                                            <fieldset class="form-group">
                                                <input name="title" type="text" class="form-control form-control-lg" minlength=TITLE_MIN_LENGTH
                                                    placeholder="Query Title" value=a.query.title />
                                            </fieldset>
                                            <fieldset class="form-group">
                                                <input name="description" type="text" class="form-control" minlength=DESCRIPTION_MIN_LENGTH
                                                    placeholder="What's this query about?" value=a.query.description />
                                            </fieldset>
                                            <fieldset class="form-group">
                                                <textarea name="body" class="form-control" rows="8"
                                                    placeholder="Write your query (in markdown)" minlength=BODY_MIN_LENGTH
                                                    prop:value=a.query.body.unwrap_or_default()></textarea>
                                            </fieldset>
                                            <fieldset class="form-group">
                                                <input name="tag_list" type="text" class="form-control"
                                                    placeholder="Enter tags(space separated)" value=a.query.tag_list.join(" ") />
                                            </fieldset>
                                            <input name="slug" type="hidden" value=a.query.slug />
                                            <button class="btn btn-lg pull-xs-right btn-primary" type="submit">
                                                "Submit Query"
                                            </button>
                                        </fieldset>
                                    }
                                }))}
                            </ErrorBoundary>
                        </Suspense>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
