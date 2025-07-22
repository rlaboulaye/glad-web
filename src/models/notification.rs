use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub notification_id: i64,
    pub user_id: i64,
    pub query_id: i64,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: String,
}

impl Notification {
    /// Create a new notification for a query status change
    pub async fn create_for_query_status(
        user_id: i64,
        query_id: i64,
        query_description: &str,
        status: &str,
    ) -> Result<i64, sqlx::Error> {
        let (title, message) = match status {
            "completed" => (
                "Query Completed".to_string(),
                format!("Your query '{}' has completed successfully", query_description),
            ),
            "errored" => (
                "Query Error".to_string(), 
                format!("Your query '{}' has errored", query_description),
            ),
            _ => return Err(sqlx::Error::RowNotFound), // Only create notifications for completed/errored
        };

        let result = sqlx::query!(
            "INSERT INTO notifications (user_id, query_id, title, message) VALUES ($1, $2, $3, $4)",
            user_id,
            query_id,
            title,
            message
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get all notifications for a user, ordered by most recent first
    pub async fn for_user(user_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query!(
            "SELECT notification_id, user_id, query_id, title, message, is_read, created_at 
             FROM notifications 
             WHERE user_id = $1 
             ORDER BY created_at DESC",
            user_id
        )
        .map(|row| Self {
            notification_id: row.notification_id.expect("notification_id should not be null"),
            user_id: row.user_id,
            query_id: row.query_id,
            title: row.title,
            message: row.message,
            is_read: row.is_read, // SQLite handles booleans directly
            created_at: row.created_at.to_string(),
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    /// Get count of unread notifications for a user
    pub async fn unread_count_for_user(user_id: i64) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = $1 AND is_read = FALSE",
            user_id
        )
        .fetch_one(crate::database::get_db())
        .await?;

        Ok(result.count)
    }

    /// Mark a notification as read
    pub async fn mark_as_read(notification_id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE notifications SET is_read = TRUE 
             WHERE notification_id = $1 AND user_id = $2",
            notification_id,
            user_id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_as_read(user_id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE",
            user_id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected())
    }

    /// Find queries with completed/errored status that don't have notifications yet
    /// Returns Vec<(query_id, user_id, description, status)>
    pub async fn find_queries_needing_notifications() -> Result<Vec<(i64, i64, String, String)>, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT q.query_id, q.user_id, q.description, q.status
            FROM query q
            LEFT JOIN notifications n ON q.query_id = n.query_id
            WHERE q.status IN ('completed', 'errored')
            AND n.query_id IS NULL
            "#
        )
        .map(|row| {
            let description = row.description.unwrap_or_else(|| "Untitled Query".to_string());
            (row.query_id, row.user_id, description, row.status)
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    /// Process all queries that need notifications
    pub async fn process_pending_notifications() -> Result<usize, sqlx::Error> {
        let queries = Self::find_queries_needing_notifications().await?;
        let mut created_count = 0;

        for (query_id, user_id, description, status) in queries {
            match Self::create_for_query_status(user_id, query_id, &description, &status).await {
                Ok(_) => {
                    created_count += 1;
                    tracing::info!("Created notification for query {} (status: {})", query_id, status);
                }
                Err(e) => {
                    tracing::error!("Failed to create notification for query {}: {}", query_id, e);
                }
            }
        }

        if created_count > 0 {
            tracing::info!("Created {} notifications for completed/errored queries", created_count);
        }

        Ok(created_count)
    }
}