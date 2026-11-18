use crate::api::api_list_periods;
use crate::components::layout::{AppLayout, ToastContainer};
use crate::pages::admin_users::AdminUsersPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::lagging::LaggingPage;
use crate::pages::login::LoginPage;
use crate::pages::programs::ProgramsPage;
use crate::pages::report::ReportPage;
use crate::pages::work_hours::WorkHoursPage;
use crate::state::AppState;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);

    Effect::new(move |_| {
        let Some(token) = state.token.get() else {
            return;
        };
        if state.active_period.get_untracked().is_some() {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            match api_list_periods(&token).await {
                Ok(list) => {
                    if let Some(first) = list.into_iter().next() {
                        state.active_period.set(Some(first));
                    }
                }
                Err(e) => {
                    if e.contains("401") || e.contains("Token") {
                        state.logout();
                    }
                }
            }
        });
    });

    view! {
        <ToastContainer />
        <Show
            when=move || state.token.get().is_some()
            fallback=|| view! { <LoginPage /> }
        >
            <Router>
                <AppLayout>
                    <Routes fallback=|| view! { <DashboardPage /> }>
                        <Route path=path!("/sobat-k3") view=DashboardPage />
                        <Route path=path!("/sobat-k3/") view=DashboardPage />
                        <Route path=path!("/sobat-k3/work-hours") view=WorkHoursPage />
                        <Route path=path!("/sobat-k3/programs") view=ProgramsPage />
                        <Route path=path!("/sobat-k3/lagging") view=LaggingPage />
                        <Route path=path!("/sobat-k3/report") view=ReportPage />
                        <Route path=path!("/sobat-k3/admin") view=AdminUsersPage />
                    </Routes>
                </AppLayout>
            </Router>
        </Show>
    }
}
