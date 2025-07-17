use argon2::{
    password_hash::{rand_core::OsRng, Error as ArgonError, SaltString},
    Argon2, PasswordHash, PasswordHasher,
};
use serde::{Deserialize, Serialize};
use tokio::task;

// Validation constants
pub const USERNAME_MIN_LENGTH: usize = 8;
pub const PASSWORD_MIN_LENGTH: usize = 8;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct User {
    username: String,
    #[serde(skip_serializing)]
    password: Option<String>,
    email: String,
    bio: Option<String>,
}

static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

impl User {
    #[inline]
    pub fn username(&self) -> String {
        self.username.to_string()
    }
    #[inline]
    pub fn email(&self) -> String {
        self.email.to_string()
    }
    #[inline]
    pub fn bio(&self) -> Option<String> {
        self.bio.clone()
    }

    /// Validate password meets minimum requirements
    pub fn validate_password(password: &str) -> Result<(), crate::models::DatabaseError> {
        if password.len() < PASSWORD_MIN_LENGTH {
            return Err(crate::models::DatabaseError::PasswordTooWeak);
        }
        Ok(())
    }

    pub fn set_password(mut self, password: String) -> Result<Self, crate::models::DatabaseError> {
        Self::validate_password(&password)?;
        self.password = Some(password);
        Ok(self)
    }

    pub fn set_username(mut self, username: String) -> Result<Self, crate::models::DatabaseError> {
        if username.len() < USERNAME_MIN_LENGTH {
            return Err(crate::models::DatabaseError::UsernameTooShort);
        }
        self.username = username;
        Ok(self)
    }

    fn validate_email(email: &str) -> bool {
        EMAIL_REGEX
            .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
            .is_match(email)
    }

    pub fn set_email(mut self, email: String) -> Result<Self, crate::models::DatabaseError> {
        if !Self::validate_email(&email) {
            return Err(crate::models::DatabaseError::InvalidEmail);
        }
        self.email = email;
        Ok(self)
    }

    pub fn set_bio(mut self, bio: String) -> Result<Self, crate::models::DatabaseError> {
        if bio.is_empty() {
            self.bio = None;
        } else {
            self.bio = Some(bio);
        }
        Ok(self)
    }

    pub async fn get(username: String) -> Result<Self, crate::models::DatabaseError> {
        sqlx::query_as!(
            Self,
            //"SELECT username, email, bio, NULL as \"password: Option<String>\" FROM user WHERE username=$1",
            "SELECT username, email, bio, password FROM user WHERE username=$1",
            username
        )
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => crate::models::DatabaseError::UserNotFound,
            _ => crate::models::DatabaseError::from(e),
        })
    }

    pub async fn get_email(email: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            //"SELECT username, email, bio, NULL as password FROM user WHERE email=$1",
            "SELECT username, email, bio, password FROM user WHERE email=$1",
            email
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    pub async fn insert(&self) -> Result<sqlx::sqlite::SqliteQueryResult, crate::models::DatabaseError> {
        let password = match &self.password {
            Some(password) => hash_password(password.to_string())
                .await
                .expect("Failed to hash password"),
            None => String::new(),
        };
        
        // Convert None bio to empty string to match database DEFAULT
        let bio = self.bio.as_deref().unwrap_or("");
        
        sqlx::query!(
            "INSERT INTO user(username, bio, email, password) VALUES ($1, $2, $3, $4)",
            self.username,
            bio,
            self.email,
            password,
        )
        .execute(crate::database::get_db())
        .await
        .map_err(crate::models::DatabaseError::from)
    }

    pub async fn update(&self) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        match &self.password {
            Some(password) => {
                let password = hash_password(password.to_string())
                    .await
                    .expect("Failed to hash password");
                sqlx::query!(
                    "UPDATE user SET bio=$2, email=$3, password=$4 WHERE username=$1",
                    self.username,
                    self.bio,
                    self.email,
                    password,
                )
                .execute(crate::database::get_db())
                .await
            }
            None => {
                sqlx::query!(
                    "UPDATE user SET bio=$2, email=$3 WHERE username=$1",
                    self.username,
                    self.bio,
                    self.email,
                )
                .execute(crate::database::get_db())
                .await
            }
        }
    }
}

// Hash a password in a separate blocking thread
async fn hash_password(password: String) -> Result<String, String> {
    task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e: ArgonError| e.to_string()) // Convert error to String
    })
    .await
    .map_err(|e| e.to_string())? // Flatten Result<Result<String, String>, JoinError>
}

pub async fn verify_password(password: String, password_hash: String) -> Result<(), String> {
    Ok(task::spawn_blocking(move || -> Result<(), String> {
        let hash =
            PasswordHash::new(&password_hash).map_err(|e| format!("invalid password hash: {e}"))?;

        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|_| "password did not match".to_string())
    })
    .await
    .map_err(|e| e.to_string())??)
}
