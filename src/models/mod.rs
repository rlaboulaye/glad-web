mod query;
mod user;
pub use query::{Cohort, Query};
#[cfg(feature = "ssr")]
pub use user::verify_password;
pub use user::User;

#[cfg(feature = "ssr")]
const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
