use leptos::{leptos_dom, prelude::*};
use leptos_meta::*;

use crate::models::{Cohort, Query};

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum FindResponse {
    ValidationError(String),
    InsertError,
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

const DESCRIPTION_MIN_LENGTH: usize = 4;

#[server(RetrieveCohortsAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn retrieve_cohorts() -> Result<Vec<Cohort>, ServerFnError> {
    Cohort::retrieve_all().await.map_err(move |x| {
        tracing::error!(
            "Problem while retrieving cohorts with error {}",
            x.to_string()
        );
        ServerFnError::ServerError("Problem while retrieving cohorts".into())
    })
}

#[server(FindAction, "/api")]
#[tracing::instrument]
pub async fn find_action(
    description: String,
    #[server(default)] self_described_latino: String,
    n_controls: usize,
    #[server(default)] cohorts: Vec<String>,
) -> Result<FindResponse, ServerFnError> {
    let Some(_) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Ok(FindResponse::ValidationError(
            "you should be authenticated".to_string(),
        ));
    };
    match Query::insert(
        description,
        match self_described_latino.trim() {
            "on" => true,
            _ => false,
        },
        n_controls,
        cohorts,
    )
    .await
    {
        Ok(query_id) => {
            leptos_axum::redirect(&format!("/query/{query_id}"));
            Ok(FindResponse::Successful(query_id.to_string()))
        }
        Err(x) => {
            tracing::error!("FIND CONTROLS ERROR: {}", x.to_string());
            Ok(FindResponse::InsertError)
        }
    }
}

#[tracing::instrument]
#[component]
pub fn Find() -> impl IntoView {
    let find_server_action: ServerAction<FindAction> = ServerAction::new();
    let cohorts_resource = Resource::new(move || (), async move |_| retrieve_cohorts().await);
    let selected_cohorts = RwSignal::new(Vec::<Cohort>::new());
    let result = find_server_action.value();
    let error = move || {
        result.with(|x| {
            x.as_ref().map_or(true, |y| {
                y.is_err() || !matches!(y, Ok(FindResponse::Successful(_)))
            })
        })
    };

    view! {
        <Title text="Find Controls"/>
        <div class="find-page">
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
                                    Ok(FindResponse::ValidationError(x)) => {
                                        format!("Problem while validating: {x}")
                                    }
                                    Ok(FindResponse::InsertError) => {
                                        "Error while submitting the query, please, try again later".into()
                                    }
                                    Ok(FindResponse::Successful(_)) => {
                                        String::new()
                                    }
                                    Err(x) => format!("Unexpected error: {x}"),
                                }
                            })}
                        </strong>
                    </p>

                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=find_server_action>
                            <fieldset>
                                <fieldset class="form-group">
                                    <input name="description" type="text" class="form-control" minlength=DESCRIPTION_MIN_LENGTH
                                        placeholder="Give a brief description of your project and need for additional controls." />
                                </fieldset>
                                <fieldset class="form-group">
                                    <label for="self_described_latino">"Restrict search to self-described latinos only"</label>
                                    <input name="self_described_latino" type="checkbox" class="form-control" value="on" />
                                </fieldset>
                                <h3>"How many controls would you like to find?"</h3>
                                <fieldset class="form-group">
                                    <input name="n_controls" type="number" class="form-control" value=100 />
                                </fieldset>
                                <h3>"Select cohorts to exclude from matching procedure:"</h3>
                                <div class="selected-cohorts">
                                    //
                                    <div class="scrollable-container overflow-y-auto max-h-64">
                                        <Suspense fallback=move || view! {<p>"Loading Cohorts"</p> }>
                                            <ErrorBoundary fallback=|_| {
                                                view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
                                            }>
                                                //view! {
                                                {
                                                    move || {
                                                        cohorts_resource.get().map(move |cohorts_result| {
                                                            cohorts_result.map(move |cohorts| {
                                                                view! {
                                                                    <For
                                                                        each=move || cohorts.clone().into_iter().enumerate()
                                                                        key=|(i, _)| *i
                                                                        children=move |(_, cohort): (usize, Cohort)| {
                                                                            view! {
                                                                                <div class="checkbox-item">
                                                                                    <input type="checkbox" value=cohort.cohort_name.clone() name="cohorts[]" />
                                                                                    <label for="cohorts[]">{cohort.cohort_name.clone()}</label>
                                                                                </div>
                                                                            }
                                                                        }
                                                                    />
                                                                }
                                                            })
                                                        })
                                                    }
                                                }
                                            </ErrorBoundary>
                                        </Suspense>
                                    </div>
                                </div>
                                <button class="btn btn-lg pull-xs-right btn-primary" type="submit">
                                    "Submit Query"
                                </button>
                            </fieldset>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
