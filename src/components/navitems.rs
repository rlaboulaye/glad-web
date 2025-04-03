use crate::auth::*;
use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub(crate) fn NavItems(logout: LogoutSignal, user: UserSignal) -> impl IntoView {
    let profile_label = move || user.get().unwrap_or_default().1;
    let profile_href = move || format!("/profile/{}", profile_label());

    view! {
        <li class="nav-item">
            <A class:nav-link=true href="/" exact=true><i class="ion-home"></i>" Home"</A>
        </li>
        <Show when=move || user.with(Option::is_none) fallback=move || {
            view!{
                <li class="nav-item">
                    <A class:nav-link=true href="/editor"><i class="ion-compose"></i>" New Article"</A>
                </li>
                <li class="nav-item">
                    <A class:nav-link=true href="/settings"><i class="ion-gear-a"></i>" Settings"</A>
                </li>
                <li class="nav-item">
                    <A class:nav-link=true href=profile_href><i class="ion-person"></i>" "{profile_label}</A>
                </li>
                <li class="nav-item">
                    <ActionForm action=logout>
                        <button class="nav-link" style="background: none; border: none;">
                            <i class="ion-log-out"></i>" Logout"
                        </button>
                    </ActionForm>
                </li>
            }
        }>
            <li class="nav-item">
                <A class:nav-link=true href="/signup"><i class="ion-plus-round"></i>" Sign up"</A>
            </li>
            <li class="nav-item">
                <A class:nav-link=true href="/login"><i class="ion-log-in"></i>" Login"</A>
            </li>
        </Show>
    }
}
