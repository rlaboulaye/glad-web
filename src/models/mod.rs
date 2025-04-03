mod user;
pub use user::{hash_password, User};
mod pagination;
pub use pagination::Pagination;
mod query;
pub use query::Query;

#[cfg(feature = "ssr")]
const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
