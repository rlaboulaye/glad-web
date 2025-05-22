use crate::auth::*;
use leptos::prelude::*;
use leptos_router::{components::*, hooks::use_location, location::Location};

#[derive(Clone, PartialEq)]
enum MenuID {
    User,
    Mobile,
}

// #[component]
// fn MobileMenu(
//     active_menu: RwSignal<Option<MenuID>>,
//     username: UsernameSignal,
//     pathname: &Signal<String>,
// ) -> impl IntoView {
//     let class = move || match active_menu.get() {
//         Some(MenuID::Mobile) => "sm:hidden",
//         _ => "sm:block",
//     };
//     view! {
//         <div class=class id="mobile-menu">
//             <div class="space-y-1 px-2 pb-3 pt-2">
//                 <Show when=move || username.with(Option::is_none) fallback=move || {
//                     view! {
//                         <NavLink href="/dashboard" mobile={ true } pathname=pathname> "Dashboard" </NavLink>
//                         <NavLink href="/find" mobile={ true } pathname=pathname> "Find Controls" </NavLink>
//                     }
//                 }>
//                     <NavLink href="/signup" mobile={ true } pathname=pathname> "Sign up" </NavLink>
//                     <NavLink href="/login" mobile={ true } pathname=pathname> "Login" </NavLink>
//                 </Show>
//                 <NavLink href="/explore" mobile={ true } pathname=pathname> "Explore" </NavLink>
//             </div>
//         </div>
//     }
// }

#[component]
fn MobileMenu(active_menu: RwSignal<Option<MenuID>>, username: UsernameSignal) -> impl IntoView {
    let class = move || {
        if active_menu.get() == Some(MenuID::Mobile) {
            "sm:hidden"
        } else {
            "hidden"
        }
    };
    view! {
        <div class=class id="mobile-menu">
            <div class="space-y-1 px-2 pb-3 pt-2">
                <Show when=move || username.with(Option::is_none) fallback=move || {
                    view! {
                        <MobileNavLink href="/dashboard"> "Dashboard" </MobileNavLink>
                        <MobileNavLink href="/find"> "Find Controls" </MobileNavLink>
                    }
                }>
                    <MobileNavLink href="/signup"> "Sign up" </MobileNavLink>
                    <MobileNavLink href="/login"> "Login" </MobileNavLink>
                </Show>
                <MobileNavLink href="/explore"> "Explore" </MobileNavLink>
            </div>
        </div>
    }
}

#[component]
fn UserMenu(active_menu: RwSignal<Option<MenuID>>, logout: LogoutSignal) -> impl IntoView {
    let class_styling = String::from("absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black/5 focus:outline-none");
    let class = move || {
        if active_menu.get() == Some(MenuID::User) {
            format!("block {}", class_styling)
        } else {
            format!("hidden {}", class_styling)
        }
    };
    view! {
        <div
            class=class
            role="menu"
            aria-orientation="vertical"
            aria-labelledby="user-menu-button"
            tabindex="-1"
        >
            // <!-- Active: "bg-gray-100 outline-none", Not Active: "" -->
            <A href="/settings">
                <div
                    class="block px-4 py-2 text-sm text-gray-700"
                    role="menuitem"
                    tabindex="-1"
                    id="user-menu-item-0"
                >
                    Settings
                </div>
            </A>
            <ActionForm action=logout>
               <button
                   class="block px-4 py-2 text-sm text-gray-700"
                   role="menuitem"
                   tabindex="-1"
                   id="user-menu-item-1"
               >
                   Logout
               </button>
            </ActionForm>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, children: Children) -> impl IntoView {
    let location = use_location();
    let is_active = move || location.pathname.get().starts_with(href);

    let mut class_prefix = String::from("rounded-md px-3 py-2 font-medium text-sm");
    let class = move || {
        if is_active() {
            format!("{} bg-gray-900 text-white", class_prefix)
        } else {
            format!(
                "{} text-gray-300 hover:bg-gray-700 hover:text-white",
                class_prefix
            )
        }
    };

    view! {
        <A href=href>
            <div class=class aria-current=move || if is_active() { Some("page") } else { None }>
                { children() }
            </div>
        </A>
    }
}

#[component]
fn MobileNavLink(href: &'static str, children: Children) -> impl IntoView {
    view! {
        <A href=href>
            <div class="rounded-md px-3 py-2 font-medium block text-base text-gray-300 hover:bg-gray-700 hover:text-white">
                { children() }
            </div>
        </A>
    }
}

#[component]
pub(crate) fn NavItems(logout: LogoutSignal, username: UsernameSignal) -> impl IntoView {
    let profile_label = move || username.get().unwrap_or_default();
    let active_menu: RwSignal<Option<MenuID>> = RwSignal::new(None);
    let toggle_menu = move |menu_id: MenuID| {
        active_menu.update(move |current| {
            if *current == Some(menu_id.clone()) {
                *current = None;
            } else {
                *current = Some(menu_id);
            }
        });
    };

    view! {
        <nav class="bg-gray-800">
            <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
                <div class="relative flex h-16 items-center justify-between">
                    <div class="absolute inset-y-0 left-0 flex items-center sm:hidden">
                        <button
                            type="button"
                            on:click= move |_| toggle_menu(MenuID::Mobile)
                        >
                            "BUTTON"
                            // <svg
                            //     class="block size-6"
                            //     fill="none"
                            //     viewBox="0 0 24 24"
                            //     stroke-width="1.5"
                            //     stroke="currentColor"
                            //     aria-hidden="true"
                            //     data-slot="icon"
                            // >
                            //     <path
                            //         stroke-linecap="round"
                            //         stroke-linejoin="round"
                            //         d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
                            //     />
                            // </svg>
                        </button>
                        // <!-- Mobile menu button-->
                        // <button
                        //     type="button"
                        //     // on:click= move |_| toggle_menu(MenuID::Mobile)
                        //     class="relative inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                        //     aria-controls="mobile-menu"
                        //     aria-expanded="false"
                        // >
                        //     <span class="absolute -inset-0.5"></span>
                        //     <span class="sr-only">Open main menu</span>
                        //     // <!--
                        //     // Icon when menu is closed.
                        //     //
                        //     // Menu open: "hidden", Menu closed: "block"
                        //     // -->
                        //     <svg
                        //         class="block size-6"
                        //         fill="none"
                        //         viewBox="0 0 24 24"
                        //         stroke-width="1.5"
                        //         stroke="currentColor"
                        //         aria-hidden="true"
                        //         data-slot="icon"
                        //     >
                        //         <path
                        //             stroke-linecap="round"
                        //             stroke-linejoin="round"
                        //             d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
                        //         />
                        //     </svg>
                        //     // <!--
                        //     // Icon when menu is open.
                        //     //
                        //     // Menu open: "block", Menu closed: "hidden"
                        //     // -->
                        //     <svg
                        //         class="hidden size-6"
                        //         fill="none"
                        //         viewBox="0 0 24 24"
                        //         stroke-width="1.5"
                        //         stroke="currentColor"
                        //         aria-hidden="true"
                        //         data-slot="icon"
                        //     >
                        //         <path
                        //             stroke-linecap="round"
                        //             stroke-linejoin="round"
                        //             d="M6 18 18 6M6 6l12 12"
                        //         />
                        //     </svg>
                        // </button>
                    </div>
                    <div class="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start">
                        <div class="flex shrink-0 items-center">
                            <A href="/" exact=true>
                                <p class="px-3 py-1 text-lg font-bold text-green-300">
                                    "glad"
                                </p>
                            </A>
                        </div>
                        <div class="hidden sm:ml-6 sm:block">
                            <div class="flex space-x-4">
                                <Show when=move || username.with(Option::is_none) fallback=move || {
                                    view! {
                                        <NavLink href="/dashboard"> "Dashboard" </NavLink>
                                        <NavLink href="/find"> "Find Controls" </NavLink>
                                    }
                                }>
                                    <NavLink href="/signup"> "Sign up" </NavLink>
                                    <NavLink href="/login"> "Login" </NavLink>
                                </Show>
                                <NavLink href="/explore"> "Explore" </NavLink>
                            </div>
                        </div>
                    </div>
                    <Show when=move || username.with(Option::is_none) fallback=move || {
                        view! {
                            <div class="absolute inset-y-0 right-0 flex items-center pr-2 sm:static sm:inset-auto sm:ml-6 sm:pr-0">
                                <button
                                    type="button"
                                    class="relative rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
                                >
                                    <span class="absolute -inset-1.5"></span>
                                    <span class="sr-only">View notifications</span>
                                    <svg
                                        class="size-6"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke-width="1.5"
                                        stroke="currentColor"
                                        aria-hidden="true"
                                        data-slot="icon"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0"
                                        />
                                    </svg>
                                </button>

                                // <!-- Profile dropdown -->
                                <div class="relative ml-3">
                                    <div>
                                        <button
                                            type="button"
                                            on:click= move |_| toggle_menu(MenuID::User)
                                            //class="relative flex rounded-full bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
                                            class="rounded-md px-3 py-2 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white"
                                            id="user-menu-button"
                                            aria-expanded="false"
                                            aria-haspopup="true"
                                        >
                                            <span class="absolute -inset-1.5"></span>
                                            <span class="sr-only">Open user menu</span>
                                         //<img class="size-8 rounded-full" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
                                            {profile_label}
                                        </button>
                                    </div>
                                    <UserMenu active_menu=active_menu logout=logout />
                                </div>
                            </div>
                        }
                    }>
                        <p></p>
                    </Show>
                </div>
            </div>
            // <!-- Mobile menu, show/hide based on menu state. -->
            <MobileMenu active_menu=active_menu username=username />
        </nav>
    }
}
