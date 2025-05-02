use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    StaticSegment,
};

use crate::auth::{current_user, LoginSignal, LogoutSignal, SignupSignal, UsernameSignal};
use crate::components::NavItems;
use crate::routes::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let username: UsernameSignal = RwSignal::new(None);

    let login: LoginSignal = ServerAction::new();
    let logout: LogoutSignal = ServerAction::new();
    let signup: SignupSignal = ServerAction::new();

    let (logout_version, login_version, signup_version) =
        (logout.version(), login.version(), signup.version());

    let user = Resource::new(
        move || {
            (
                logout_version.get(),
                login_version.get(),
                signup_version.get(),
            )
        },
        move |_| {
            tracing::debug!("fetch user");
            current_user()
        },
    );

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/glad-web.css" />

        // sets the document title
        <Title text="GLAD" />

        // content for this welcome page
        <Router>
            <Transition fallback=|| {
                view! { <p>"Loading Navigation bar"</p> }
            }>
                {move || {
                    user.get()
                        .map(move |x| {
                            username.set(x.map(|y| y.username()).ok());
                            view! { <NavItems logout username /> }
                        })
                }}
            </Transition>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route
                        path=StaticSegment("")
                        view=move || {
                            view! {
                                <Transition fallback=|| view! { <p>"Loading HomePage"</p> }>
                                    <HomePage />
                                </Transition>
                            }
                        }
                    />
                    <Route path=StaticSegment("find") view=move || view! { <Find /> } />
                    <Route path=StaticSegment("login") view=move || view! { <Login login /> } />
                    <Route
                        path=StaticSegment("reset_password")
                        view=move || view! { <ResetPassword /> }
                    />
                    <Route path=StaticSegment("signup") view=move || view! { <Signup signup /> } />
                    <Route
                        path=StaticSegment("settings")
                        view=move || view! { <Settings logout /> }
                    />
                </Routes>
            </main>
        </Router>
    }
}
