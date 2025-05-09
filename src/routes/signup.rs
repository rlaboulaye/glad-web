use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;

use crate::auth::{validate_signup, SignupAction, SignupResponse, SignupSignal};

#[component]
pub fn Signup(signup: SignupSignal) -> impl IntoView {
    let result_of_call = signup.value();

    let error_cb = move || {
        result_of_call
            .get()
            .map(|msg| match msg {
                Ok(SignupResponse::ValidationError(x)) => format!("Problem while validating: {x}"),
                Ok(SignupResponse::CreateUserError(x)) => {
                    format!("Problem while creating user: {x}")
                }
                Ok(SignupResponse::Success) => {
                    tracing::info!("Signup success! redirecting");
                    "Done".into()
                }
                Err(x) => {
                    tracing::error!("Problem during signup: {x:?}");
                    "There was a problem, try again later".into()
                }
            })
            .unwrap_or_default()
    };

    view! {
        <Title text="Sign up" />
        <div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-md mt-20">
                <h2 class="text-center text-3xl font-bold tracking-tight text-gray-800 dark:text-gray-100">
                    Sign up
                </h2>

                <p class="mt-2 text-center text-sm text-gray-600 dark:text-gray-400">
                    <A
                        href="/login"
                        class:text-green-400=true
                        class:dark:text-green-300=true
                        class:hover:underline=true
                    >
                        Have an account?
                    </A>
                </p>

                <p class="text-center text-sm text-red-500 mt-2">
                    {error_cb}
                </p>

                <ActionForm
                    action=signup
                    on:submit=move |ev| {
                        let Ok(data) = SignupAction::from_event(&ev) else {
                            return ev.prevent_default();
                        };
                        if let Err(x) = validate_signup(
                            data.username,
                            data.bio,
                            data.email,
                            data.password,
                        ) {
                            result_of_call.set(Some(Ok(SignupResponse::ValidationError(x))));
                            ev.prevent_default();
                        }
                    }
                    class:p-8=true
                    class:shadow-md=true
                    class:rounded-lg=true
                    class:mt-8=true
                    class:space-y-6=true
                >
                    <div>
                        <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Username
                        </label>
                        <input
                            name="username"
                            type="text"
                            placeholder="Your Username"
                            required=true
                            class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                        />
                    </div>

                    <div>
                        <label for="bio" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Bio
                        </label>
                        <textarea
                            name="bio"
                            rows="6"
                            placeholder="Short bio about you and your work"
                            class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                        ></textarea>
                    </div>

                    <div>
                        <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Email
                        </label>
                        <input
                            name="email"
                            type="email"
                            placeholder="Email"
                            required=true
                            class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                        />
                    </div>

                    <div>
                        <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            Password
                        </label>
                        <input
                            name="password"
                            type="password"
                            placeholder="Password"
                            required=true
                            class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                        />
                    </div>

                    <div>
                        <button
                            type="submit"
                            class="w-full flex justify-center rounded-md border border-transparent bg-green-400 hover:bg-green-500 px-4 py-2 text-sm font-medium text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-green-300 focus:ring-offset-2"
                        >
                            Sign up
                        </button>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
