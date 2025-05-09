use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;

use crate::auth::{LoginMessages, LoginSignal};

#[component]
pub fn Login(login: LoginSignal) -> impl IntoView {
    let result_of_call = login.value();

    let error = move || {
        result_of_call.with(|msg| {
            msg.as_ref()
                .map(|inner| match inner {
                    Ok(LoginMessages::Unsuccessful) => "Incorrect user or password",
                    Ok(LoginMessages::Successful) => {
                        tracing::info!("login success!");
                        "Done"
                    }
                    Err(x) => {
                        tracing::error!("Problem during login: {x:?}");
                        "There was a problem, try again later"
                    }
                })
                .unwrap_or_default()
        })
    };

    view! {
    <Title text="Login" />
    <div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md mt-20">
            <h2 class="text-center text-3xl font-bold tracking-tight text-gray-800 dark:text-gray-100">
                Login
            </h2>

            <p class="mt-2 text-center text-sm text-red-500">
                {error}
            </p>
            <ActionForm
                action=login
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
                        class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white"
                    />
                </div>

                <div class="flex justify-between items-center text-sm mb-4">
                    <A
                        href="/reset_password"
                        class:text-green-400=true
                        class:dark:text-green-300=true
                        class:hover:underline=true
                    >
                        Reset password
                    </A>
                </div>

                <div>
                    <button
                        type="submit"
                        class="w-full flex justify-center rounded-md border border-transparent bg-green-400 hover:bg-green-500 px-4 py-2 text-sm font-medium text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-green-300 focus:ring-offset-2"
                    >
                        Sign in
                    </button>
                </div>
            </ActionForm>
        </div>
    </div>
    }
}
