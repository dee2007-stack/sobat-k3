use crate::api::api_login;
use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::icons::IconShieldCheck;
use crate::components::ui::input::{Input, Label};
use crate::state::AppState;
use leptos::prelude::*;
use shared::models::LoginRequest;

#[component]
pub fn LoginPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let u = username.get_untracked();
        let p = password.get_untracked();
        if u.trim().is_empty() || p.is_empty() {
            error_msg.set(Some("Username dan password wajib diisi".to_string()));
            return;
        }
        loading.set(true);
        error_msg.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let req = LoginRequest {
                username: u,
                password: p,
            };
            match api_login(&req).await {
                Ok(res) => {
                    state.set_auth(res.token, res.user);
                    state.notify("Berhasil masuk ke Sobat K3", false);
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-emerald-950 via-emerald-900 to-slate-900 p-4">
            <Card class="w-full max-w-md shadow-2xl border-emerald-800/30">
                <CardHeader class="text-center space-y-2 pb-2">
                    <div class="mx-auto h-12 w-12 rounded-xl bg-primary text-primary-foreground flex items-center justify-center shadow-md">
                        <IconShieldCheck class="w-6 h-6" />
                    </div>
                    <CardTitle class="text-xl pt-1">"Sobat K3 — BBPOM di Bandung"</CardTitle>
                    <p class="text-xs text-muted-foreground">
                        "Sistem Rencana & Program Kerja K3L / P2K3 Tahun 2027"
                    </p>
                </CardHeader>
                <CardContent class="pt-3">
                    <form on:submit=on_submit class="space-y-4">
                        <Show when=move || error_msg.get().is_some()>
                            <div class="rounded-lg bg-red-50 border border-red-200 px-3.5 py-2.5 text-xs text-red-700">
                                {move || error_msg.get().unwrap_or_default()}
                            </div>
                        </Show>
                        <div class="space-y-1.5">
                            <Label>"Username atau Email"</Label>
                            <Input value=username placeholder="contoh: admin / ketua_k3" />
                        </div>
                        <div class="space-y-1.5">
                            <Label>"Password"</Label>
                            <Input value=password r#type="password" placeholder="••••••••" />
                        </div>
                        <Button r#type="submit" class="w-full" disabled=Signal::derive(move || loading.get())>
                            {move || if loading.get() { "Memproses..." } else { "Masuk ke Sistem" }}
                        </Button>
                    </form>
                    <div class="mt-5 pt-4 border-t text-[11px] text-muted-foreground space-y-1.5">
                        <div class="font-semibold text-foreground">"Akun Demo (Password: SobatK3!2027):"</div>
                        <div class="grid grid-cols-2 gap-1.5">
                            <span>"• admin (Admin)"</span>
                            <span>"• ketua_k3 (Ketua)"</span>
                            <span>"• sekretaris_k3 (Sekretaris)"</span>
                            <span>"• anggota_k3 (Anggota)"</span>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}
