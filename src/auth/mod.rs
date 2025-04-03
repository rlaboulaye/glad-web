use leptos::prelude::{Action, RwSignal, ServerAction, ServerFnError};
mod api;
#[cfg(feature = "ssr")]
mod server;
pub use api::*;
#[cfg(feature = "ssr")]
pub use server::*;

//pub type LogoutSignal = Action<LogoutAction, Result<(), ServerFnError>>;
pub type LogoutSignal = ServerAction<LogoutAction>;
//pub type LoginSignal = Action<LoginAction, Result<LoginMessages, ServerFnError>>;
pub type LoginSignal = ServerAction<LoginAction>;
//pub type SignupSignal = Action<SignupAction, Result<SignupResponse, ServerFnError>>;
pub type SignupSignal = ServerAction<SignupAction>;
pub type UserSignal = RwSignal<Option<(i64, String)>>;
