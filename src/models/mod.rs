mod error;
mod query;
mod user;

pub use error::DatabaseError;
pub use query::{Cohort, Query};
pub use user::{User, verify_password};

const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
