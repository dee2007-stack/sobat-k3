use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use shared::models::{DocumentPeriodDto, UserDto};

const TOKEN_KEY: &str = "sobat_k3_token";
const USER_KEY: &str = "sobat_k3_user";

#[derive(Clone, Debug)]
pub struct ToastMessage {
    pub id: u64,
    pub title: String,
    pub is_error: bool,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub token: RwSignal<Option<String>>,
    pub user: RwSignal<Option<UserDto>>,
    pub active_period: RwSignal<Option<DocumentPeriodDto>>,
    pub toasts: RwSignal<Vec<ToastMessage>>,
    pub toast_seq: RwSignal<u64>,
}

impl AppState {
    pub fn new() -> Self {
        let saved_token = LocalStorage::get::<String>(TOKEN_KEY).ok();
        let saved_user = LocalStorage::get::<UserDto>(USER_KEY).ok();
        Self {
            token: RwSignal::new(saved_token),
            user: RwSignal::new(saved_user),
            active_period: RwSignal::new(None),
            toasts: RwSignal::new(Vec::new()),
            toast_seq: RwSignal::new(0),
        }
    }

    pub fn set_auth(&self, token: String, user: UserDto) {
        let _ = LocalStorage::set(TOKEN_KEY, &token);
        let _ = LocalStorage::set(USER_KEY, &user);
        self.token.set(Some(token));
        self.user.set(Some(user));
    }

    pub fn logout(&self) {
        LocalStorage::delete(TOKEN_KEY);
        LocalStorage::delete(USER_KEY);
        self.token.set(None);
        self.user.set(None);
        self.active_period.set(None);
    }

    pub fn notify(&self, title: impl Into<String>, is_error: bool) {
        let id = self.toast_seq.get_untracked() + 1;
        self.toast_seq.set(id);
        let msg = ToastMessage {
            id,
            title: title.into(),
            is_error,
        };
        self.toasts.update(|list| list.push(msg));
        let toasts = self.toasts;
        wasm_bindgen_futures::spawn_local(async move {
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                if let Some(w) = web_sys::window() {
                    let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 3500);
                }
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            toasts.update(|list| list.retain(|t| t.id != id));
        });
    }
}
