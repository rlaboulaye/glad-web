mod user;
pub use user::User;
#[cfg(feature = "ssr")]
pub use user::{hash_password, verify_password};

#[cfg(feature = "ssr")]
const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
