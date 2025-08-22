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
        query_title: &str,
        status: &str,
    ) -> Result<i64, sqlx::Error> {
        let (title, message) = match status {
            "completed" => (
                "Query Completed".to_string(),
                format!("Your query '{}' has completed successfully.", query_title),
            ),
            "failed" => (
                "Query Error".to_string(),
                format!("Your query '{}' has failed.", query_title),
            ),
            _ => return Err(sqlx::Error::RowNotFound), // Only create notifications for completed/failed
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

        let notification_id = result.last_insert_rowid();

        // Send email notification if user has it enabled
        if let Err(e) = Self::send_email_notification(user_id, query_id, &title, &message).await {
            tracing::error!("Failed to send email notification: {}", e);
            // Don't fail the notification creation if email fails
        }

        Ok(notification_id)
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
            notification_id: row
                .notification_id
                .expect("notification_id should not be null"),
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
    /// Returns Vec<(query_id, user_id, title, status)>
    pub async fn find_queries_needing_notifications(
    ) -> Result<Vec<(i64, i64, String, String)>, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT q.query_id, q.user_id, q.title, q.user_visible_status
            FROM query q
            LEFT JOIN notifications n ON q.query_id = n.query_id
            WHERE q.user_visible_status IN ('completed', 'failed')
            AND n.query_id IS NULL
            "#
        )
        .map(|row| {
            (
                row.query_id,
                row.user_id,
                row.title,
                row.user_visible_status,
            )
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    /// Process all queries that need notifications
    pub async fn process_pending_notifications() -> Result<usize, sqlx::Error> {
        let queries = Self::find_queries_needing_notifications().await?;
        let mut created_count = 0;

        for (query_id, user_id, title, status) in queries {
            match Self::create_for_query_status(user_id, query_id, &title, &status).await {
                Ok(_) => {
                    created_count += 1;
                    tracing::info!(
                        "Created notification for query {} (status: {})",
                        query_id,
                        status
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to create notification for query {}: {}",
                        query_id,
                        e
                    );
                }
            }
        }

        if created_count > 0 {
            tracing::info!(
                "Created {} notifications for completed/failed queries",
                created_count
            );
        }

        Ok(created_count)
    }

    /// Send email notification if user has email notifications enabled
    async fn send_email_notification(
        user_id: i64,
        query_id: i64,
        title: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::env;

        // Get user information including email preferences
        let user_info = sqlx::query!(
            "SELECT username, email, email_notifications FROM user WHERE user_id = $1",
            user_id
        )
        .fetch_one(crate::database::get_db())
        .await?;

        // Only send email if user has enabled email notifications
        if !user_info.email_notifications {
            return Ok(());
        }

        // Get email configuration from environment
        let mailer_email = env::var("MAILER_EMAIL")?;
        let mailer_passwd = env::var("MAILER_PASSWD")?;
        let smtp_server = env::var("MAILER_SMTP_SERVER")?;
        let site_base_url = env::var("SITE_BASE_URL")?;

        // Build email message
        let query_url = format!("{}/dashboard/query/{}", site_base_url, query_id);

        let email_body = format!(
            "{}\n\nView your query details: {}\n\nThis is an automated notification from GLAD.",
            message, query_url
        );

        // Extract status from title for cleaner subject line
        let status = if title == "Query Completed" {
            "Completed"
        } else {
            "Failed"
        };

        let email_message = mail_send::mail_builder::MessageBuilder::new()
            .from(("GLAD", mailer_email.as_str()))
            .to(vec![(
                user_info.username.as_str(),
                user_info.email.as_str(),
            )])
            .subject(format!("GLAD - Query Status: {}", status))
            .text_body(email_body);

        // Send email
        let mut client = mail_send::SmtpClientBuilder::new(smtp_server.as_str(), 587)
            .implicit_tls(false)
            .credentials((mailer_email.as_str(), mailer_passwd.as_str()))
            .connect()
            .await?;

        client.send(email_message).await?;

        tracing::info!(
            "Email notification sent to user {} for query {}",
            user_info.username,
            query_id
        );
        Ok(())
    }
}
