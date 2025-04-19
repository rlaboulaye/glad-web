use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Cohort {
    pub cohort_id: i64,
    pub cohort_name: String,
}

impl Cohort {
    #[cfg(feature = "ssr")]
    pub async fn retrieve_all() -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query!("SELECT cohort_id, cohort_name FROM cohort")
            .map(|x| Self {
                cohort_id: x.cohort_id,
                cohort_name: x.cohort_name,
            })
            .fetch_all(crate::database::get_db())
            .await
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Query {
    pub query_id: i64,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub self_described_latino: bool,
    pub n_controls: usize,
    pub status: String,
    pub created_at: String,
    pub status_updated_at: String,
}

impl Query {
    #[cfg(feature = "ssr")]
    pub async fn insert(
        description: String,
        self_described_latino: bool,
        n_controls: usize,
        excluded_cohorts: Vec<String>,
    ) -> Result<i64, sqlx::Error> {
        let self_described_latino = self_described_latino as i32;
        let n_controls = n_controls as i32;
        let logged_user = crate::auth::get_username();
        let file_path = "file_path".to_string();
        let logged_user_id =
            sqlx::query!("SELECT user_id FROM user WHERE username=$1", logged_user)
                .map(|x| x.user_id)
                .fetch_one(crate::database::get_db())
                .await
                .expect("Could not retrieve user_id using username {logged_user}");
        let cohort_query_params = format!("?{}", ", ?".repeat(excluded_cohorts.len() - 1));
        let cohort_query_str = format!(
            "SELECT cohort_id FROM cohort WHERE cohort_name IN ( { } )",
            cohort_query_params
        );
        let mut cohort_query = sqlx::query_scalar(&cohort_query_str);
        for cohort_name in excluded_cohorts {
            cohort_query = cohort_query.bind(cohort_name);
        }
        let excluded_cohort_ids: Vec<i32> = cohort_query
            .fetch_all(crate::database::get_db())
            .await
            .expect("Could not retrieve cohort ids");
        let query_id = sqlx::query!(
            "INSERT INTO query(user_id, description, file_path, self_described_latino, n_controls) VALUES ($1, $2, $3, $4, $5)",
            logged_user_id,
            description,
            file_path,
            self_described_latino,
            n_controls,
        )
        .execute(crate::database::get_db())
        .await
        .expect("Could not submit query")
        .last_insert_rowid();
        let query_cohort_insert_str = format!(
            "INSERT INTO query_cohort(query_id, cohort_id) VALUES ({query_id}, ?){}",
            format!(", ({query_id}, ?)").repeat(excluded_cohort_ids.len() - 1)
        );
        let mut query_cohort_insert_query = sqlx::query(&query_cohort_insert_str);
        for cohort_id in excluded_cohort_ids {
            query_cohort_insert_query = query_cohort_insert_query.bind(cohort_id);
        }
        query_cohort_insert_query
            .execute(crate::database::get_db())
            .await
            .expect("Could not link query to cohorts");
        Ok(query_id)
    }

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
            "SELECT query_id, user_id, description, self_described_latino, n_controls, status, created_at, status_updated_at FROM query WHERE user_id=$1",
            logged_user_id,
        )
        .map(|x| Self {
            query_id: x.query_id,
            user_id: x.user_id,
            description: x.description,
            self_described_latino: match x.self_described_latino {
                    0 => false,
                    _ => true,
                },
            n_controls: x.n_controls as usize,
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
            "SELECT query_id, user_id, description, self_described_latino, n_controls, status, created_at, status_updated_at FROM query WHERE query_id=$1",
            query_id,
        )
        .map(|x| Self {
            query_id: x.query_id,
            user_id: x.user_id,
            description: x.description,
            self_described_latino: match x.self_described_latino {
                    0 => false,
                    _ => true,
                },
            n_controls: x.n_controls as usize,
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
