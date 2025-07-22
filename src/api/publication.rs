use axum::{extract::Query, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct CitationsQuery {
    paper_id: String,
}

#[derive(Debug, Serialize)]
pub struct CitationData {
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub venue: Option<String>,
    pub citation_count: Option<i32>,
    pub open_access_pdf: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CitationsResponse {
    pub data: Vec<CitationData>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarPaper {
    title: String,
    authors: Option<Vec<SemanticScholarAuthor>>,
    year: Option<i32>,
    venue: Option<String>,
    #[serde(rename = "citationCount")]
    citation_count: Option<i32>,
    #[serde(rename = "openAccessPdf")]
    open_access_pdf: Option<SemanticScholarOpenAccess>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarOpenAccess {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarCitation {
    #[serde(rename = "citingPaper")]
    citing_paper: SemanticScholarPaper,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarResponse {
    data: Vec<SemanticScholarCitation>,
}

pub async fn get_citations(Query(params): Query<CitationsQuery>) -> Result<Json<CitationsResponse>, StatusCode> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.semanticscholar.org/graph/v1/paper/{}/citations?fields=title,authors,year,venue,citationCount,openAccessPdf&limit=50",
        params.paper_id
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SemanticScholarResponse>().await {
                    Ok(semantic_response) => {
                        let citations: Vec<CitationData> = semantic_response.data
                            .into_iter()
                            .map(|citation| CitationData {
                                title: citation.citing_paper.title,
                                authors: citation.citing_paper.authors
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|author| author.name)
                                    .collect(),
                                year: citation.citing_paper.year,
                                venue: citation.citing_paper.venue,
                                citation_count: citation.citing_paper.citation_count,
                                open_access_pdf: citation.citing_paper.open_access_pdf
                                    .map(|pdf| pdf.url),
                            })
                            .collect();

                        let total = citations.len();
                        Ok(Json(CitationsResponse { data: citations, total }))
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}