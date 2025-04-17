#[cfg(feature = "ssr")]
use argon2::{
    password_hash::{rand_core::OsRng, Error as ArgonError, SaltString},
    Argon2, PasswordHash, PasswordHasher,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::task;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct User {
    username: String,
    #[cfg_attr(feature = "hydrate", allow(dead_code))]
    #[serde(skip_serializing)]
    password: Option<String>,
    email: String,
    bio: Option<String>,
}

#[cfg(feature = "ssr")]
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

    pub fn set_password(mut self, password: String) -> Result<Self, String> {
        static PASSWORD_MIN: usize = 8;
        if password.len() < PASSWORD_MIN {
            return Err("You need to provide a stronger password".into());
        }
        self.password = Some(password);
        Ok(self)
    }

    pub fn set_username(mut self, username: String) -> Result<Self, String> {
        static USERNAME_MIN: usize = 8;
        if username.len() < USERNAME_MIN {
            return Err(format!(
                "Username {username} is too short, at least {USERNAME_MIN} characters"
            ));
        }
        self.username = username;
        Ok(self)
    }

    #[cfg(feature = "ssr")]
    fn validate_email(email: &str) -> bool {
        EMAIL_REGEX
            .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
            .is_match(email)
    }

    #[cfg(not(feature = "ssr"))]
    fn validate_email(email: &str) -> bool {
        crate::emailRegex(email)
    }

    pub fn set_email(mut self, email: String) -> Result<Self, String> {
        if !Self::validate_email(&email) {
            return Err(format!(
                "The email {email} is invalid, provide a correct one"
            ));
        }
        self.email = email;
        Ok(self)
    }

    pub fn set_bio(mut self, bio: String) -> Result<Self, String> {
        if bio.is_empty() {
            self.bio = None;
        } else {
            self.bio = Some(bio);
        }
        Ok(self)
    }

    #[cfg(feature = "ssr")]
    pub async fn get(username: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            //"SELECT username, email, bio, NULL as \"password: Option<String>\" FROM user WHERE username=$1",
            "SELECT username, email, bio, password FROM user WHERE username=$1",
            username
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
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

    #[cfg(feature = "ssr")]
    pub async fn insert(&self) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let password = match &self.password {
            Some(password) => hash_password(password.to_string())
                .await
                .expect("Failed to hash password"),
            None => String::new(),
        };
        sqlx::query!(
            "INSERT INTO user(username, email, password) VALUES ($1, $2, $3)",
            self.username,
            self.email,
            password,
        )
        .execute(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
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
#[cfg(feature = "ssr")]
pub async fn hash_password(password: String) -> Result<String, String> {
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

#[cfg(feature = "ssr")]
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
