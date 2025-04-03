use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Query {
    pub query_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub status_updated_at: String,
}

impl Query {
    #[cfg(feature = "ssr")]
    pub async fn for_user_profile() -> Result<Vec<Self>, sqlx::Error> {
        let logged_user = crate::auth::get_username();
        let logged_user_id =
            sqlx::query!("SELECT user_id FROM user WHERE username=$1", logged_user)
                .map(|x| x.user_id)
                .fetch_one(crate::database::get_db())
                .await
                .expect("Could not retrieve user_id using username {logged_user}");
        sqlx::query!(
            "SELECT query_id, user_id, description, status, created_at, status_updated_at FROM query WHERE user_id=$1",
            logged_user_id,
        )
        .map(|x| Self {
            query_id: x.query_id,
            user_id: x.user_id,
            description: x.description,
            status: x.status,
            created_at: x.created_at.to_string(),
            status_updated_at: x.status_updated_at.to_string(),
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn for_query(query_id: i64) -> Result<Self, sqlx::Error> {
        sqlx::query!(
            "SELECT query_id, user_id, description, status, created_at, status_updated_at FROM query WHERE query_id=$1",
            query_id,
        )
        .map(|x| Self {
            query_id: x.query_id,
            user_id: x.user_id,
            description: x.description,
            status: x.status,
            created_at: x.created_at.to_string(),
            status_updated_at: x.status_updated_at.to_string(),
        })
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn delete(query_id: i64) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query!("DELETE FROM query WHERE query_id=$1", query_id)
            .execute(crate::database::get_db())
            .await
    }
}
