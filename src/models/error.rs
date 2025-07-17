#[derive(Debug)]
pub enum DatabaseError {
    // Database-level errors
    UsernameAlreadyExists,
    EmailAlreadyExists,
    UserNotFound,
    DatabaseOperationFailed(String),
    
    // Validation errors
    UsernameTooShort,
    PasswordTooWeak, 
    InvalidEmail,
    
    // Query-related errors
    QueryNotFound,
    CohortNotFound,
}

impl From<sqlx::Error> for DatabaseError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::Database(db_error) => {
                let error_msg = db_error.to_string();
                
                // Log the actual error for debugging
                tracing::debug!("Database error: {}", error_msg);
                
                // Check for UNIQUE constraint violations
                if error_msg.contains("UNIQUE constraint failed") {
                    if error_msg.contains("user.username") {
                        return DatabaseError::UsernameAlreadyExists;
                    } else if error_msg.contains("user.email") {
                        return DatabaseError::EmailAlreadyExists;
                    }
                }
                
                DatabaseError::DatabaseOperationFailed(error_msg)
            }
            _ => DatabaseError::DatabaseOperationFailed(error.to_string()),
        }
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Database-level errors
            DatabaseError::UsernameAlreadyExists => write!(f, "Username already exists"),
            DatabaseError::EmailAlreadyExists => write!(f, "Email already exists"),
            DatabaseError::UserNotFound => write!(f, "User not found"),
            DatabaseError::DatabaseOperationFailed(msg) => write!(f, "Database operation failed: {}", msg),
            
            // Validation errors - user-friendly messages
            DatabaseError::UsernameTooShort => write!(f, "Username must be at least 8 characters long"),
            DatabaseError::PasswordTooWeak => write!(f, "Password must be at least 8 characters long"),
            DatabaseError::InvalidEmail => write!(f, "Please enter a valid email address"),
            
            // Query-related errors
            DatabaseError::QueryNotFound => write!(f, "Query not found"),
            DatabaseError::CohortNotFound => write!(f, "Cohort not found"),
        }
    }
}

impl std::error::Error for DatabaseError {}