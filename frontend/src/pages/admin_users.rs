use crate::api::{
    api_create_user, api_delete_user, api_list_audit_logs, api_list_users, api_reset_password,
    api_update_user,
};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::Dialog;
use crate::components::ui::icons::{IconActivity, IconKey, IconPencil, IconPlus, IconTrash, IconUsers};
use crate::components::ui::input::{Input, Label};
use crate::state::AppState;
use leptos::prelude::*;
use shared::models::{
    AuditLogDto, CreateUserRequest, ResetPasswordRequest, UpdateUserRequest, UserDto, UserRole,
};
use uuid::Uuid;

#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let users = RwSignal::new(Vec::<UserDto>::new());
    let logs = RwSignal::new(Vec::<AuditLogDto>::new());
    let active_tab = RwSignal::new("users".to_string());

    // Create/Edit User Dialog
    let form_open = RwSignal::new(false);
    let editing_id = RwSignal::new(Option::<Uuid>::None);
    let f_username = RwSignal::new(String::new());
    let f_email = RwSignal::new(String::new());
    let f_name = RwSignal::new(String::new());
    let f_nip = RwSignal::new(String::new());
    let f_pos = RwSignal::new(String::new());
    let f_role = RwSignal::new("anggota_tim_k3".to_string());
    let f_password = RwSignal::new(String::new());
    let f_active = RwSignal::new(true);
    let saving = RwSignal::new(false);

    // Reset Password Dialog
    let pw_open = RwSignal::new(false);
    let pw_uid = RwSignal::new(Option::<Uuid>::None);
    let pw_new = RwSignal::new(String::new());

    let load = move || {
        let Some(token) = state.token.get() else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api_list_users(&token).await {
                Ok(list) => users.set(list),
                Err(e) => state.notify(e, true),
            }
            if let Ok(l) = api_list_audit_logs(&token).await {
                logs.set(l);
            }
        });
    };

    Effect::new(move |_| {
        load();
    });

    let open_new = Callback::new(move |_| {
        editing_id.set(None);
        f_username.set(String::new());
        f_email.set(String::new());
        f_name.set(String::new());
        f_nip.set(String::new());
        f_pos.set(String::new());
        f_role.set("anggota_tim_k3".to_string());
        f_password.set(String::new());
        f_active.set(true);
        form_open.set(true);
    });

    let open_edit = move |u: UserDto| {
        editing_id.set(Some(u.id));
        f_username.set(u.username);
        f_email.set(u.email);
        f_name.set(u.full_name);
        f_nip.set(u.nip);
        f_pos.set(u.position_title);
        f_role.set(u.role.as_str().to_string());
        f_active.set(u.is_active);
        form_open.set(true);
    };

    let save_user = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let role = UserRole::from_str(&f_role.get_untracked()).unwrap_or(UserRole::AnggotaTimK3);
        let eid = editing_id.get_untracked();
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let res = match eid {
                Some(id) => {
                    let req = UpdateUserRequest {
                        email: f_email.get_untracked(),
                        full_name: f_name.get_untracked(),
                        nip: f_nip.get_untracked(),
                        position_title: f_pos.get_untracked(),
                        role,
                        is_active: f_active.get_untracked(),
                    };
                    api_update_user(&id.to_string(), &req, &token).await
                }
                None => {
                    let req = CreateUserRequest {
                        username: f_username.get_untracked(),
                        email: f_email.get_untracked(),
                        full_name: f_name.get_untracked(),
                        nip: f_nip.get_untracked(),
                        position_title: f_pos.get_untracked(),
                        role,
                        password: f_password.get_untracked(),
                    };
                    api_create_user(&req, &token).await
                }
            };
            match res {
                Ok(saved_user) => {
                    if let Some(curr) = state.user.get_untracked() {
                        if curr.id == saved_user.id {
                            if let Some(tok) = state.token.get_untracked() {
                                state.set_auth(tok, saved_user);
                            }
                        }
                    }
                    form_open.set(false);
                    state.notify("Data pengguna berhasil disimpan", false);
                    load();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let save_pw = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(uid) = pw_uid.get_untracked() else {
            return;
        };
        let req = ResetPasswordRequest {
            new_password: pw_new.get_untracked(),
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api_reset_password(&uid.to_string(), &req, &token).await {
                Ok(_) => {
                    pw_open.set(false);
                    state.notify("Password berhasil direset", false);
                }
                Err(e) => state.notify(e, true),
            }
        });
    };

    let on_delete = move |id: Uuid| {
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete_user(&id.to_string(), &token).await {
                Ok(_) => {
                    users.update(|list| list.retain(|u| u.id != id));
                    state.notify("Pengguna dihapus", false);
                }
                Err(e) => state.notify(e, true),
            }
        });
    };

    view! {
        <div class="space-y-4">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
                <div>
                    <h1 class="text-xl md:text-2xl font-bold">"Menu Admin — Manajemen Pengguna & Role"</h1>
                    <p class="text-xs md:text-sm text-muted-foreground">
                        "Kelola akun Admin, Ketua Tim K3, Sekretaris Tim K3, dan Anggota Tim K3."
                    </p>
                </div>
                <Button size=ButtonSize::Sm on_click=open_new>
                    <IconPlus class="w-3.5 h-3.5" />
                    <span>"Tambah Pengguna"</span>
                </Button>
            </div>

            <div class="flex gap-2">
                <button
                    type="button"
                    class=move || if active_tab.get() == "users" { "inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-xs font-semibold bg-primary text-primary-foreground cursor-pointer" } else { "inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-xs bg-card border cursor-pointer" }
                    on:click=move |_| active_tab.set("users".to_string())
                >
                    <IconUsers class="w-3.5 h-3.5" />
                    <span>"Daftar Pengguna"</span>
                </button>
                <button
                    type="button"
                    class=move || if active_tab.get() == "logs" { "inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-xs font-semibold bg-primary text-primary-foreground cursor-pointer" } else { "inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-xs bg-card border cursor-pointer" }
                    on:click=move |_| active_tab.set("logs".to_string())
                >
                    <IconActivity class="w-3.5 h-3.5" />
                    <span>"Audit Log Aktivitas"</span>
                </button>
            </div>

            <Show when=move || active_tab.get() == "users">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <For
                        each=move || users.get()
                        key=|u| (
                            u.id,
                            u.full_name.clone(),
                            u.email.clone(),
                            u.nip.clone(),
                            u.position_title.clone(),
                            u.role.as_str().to_string(),
                            u.is_active,
                        )
                        children=move |u| {
                            let u_edit = u.clone();
                            let uid = u.id;
                            let status_v = if u.is_active { BadgeVariant::Success } else { BadgeVariant::Destructive };
                            view! {
                                <Card>
                                    <CardHeader class="pb-2">
                                        <div class="flex items-start justify-between gap-2">
                                            <div>
                                                <CardTitle class="text-sm">{u.full_name.clone()}</CardTitle>
                                                <div class="text-xs text-muted-foreground">{format!("@{} • {}", u.username, u.email)}</div>
                                            </div>
                                            <div class="flex gap-1">
                                                <Badge variant=BadgeVariant::Default>{u.role.display_name()}</Badge>
                                                <Badge variant=status_v>{if u.is_active { "Aktif" } else { "Nonaktif" }}</Badge>
                                            </div>
                                        </div>
                                    </CardHeader>
                                    <CardContent class="text-xs space-y-2">
                                        <div class="text-muted-foreground">
                                            {format!("NIP: {} | Jabatan: {}", if u.nip.is_empty() { "-" } else { &u.nip }, u.position_title)}
                                        </div>
                                        <div class="flex flex-wrap justify-end gap-2 pt-3 border-t">
                                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(move |_| open_edit(u_edit.clone()))>
                                                <IconPencil class="w-3 h-3" />
                                                <span>"Edit"</span>
                                            </Button>
                                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=Callback::new(move |_| {
                                                pw_uid.set(Some(uid));
                                                pw_new.set(String::new());
                                                pw_open.set(true);
                                            })>
                                                <IconKey class="w-3 h-3" />
                                                <span>"Reset Password"</span>
                                            </Button>
                                            <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm on_click=Callback::new(move |_| on_delete(uid))>
                                                <IconTrash class="w-3 h-3" />
                                                <span>"Hapus"</span>
                                            </Button>
                                        </div>
                                    </CardContent>
                                </Card>
                            }
                        }
                    />
                </div>
            </Show>

            <Show when=move || active_tab.get() == "logs">
                <Card class="overflow-hidden">
                    <div class="overflow-x-auto">
                        <table class="w-full text-xs border-collapse">
                            <thead>
                                <tr class="bg-muted text-left border-b">
                                    <th class="p-2.5">"Waktu"</th>
                                    <th class="p-2.5">"User"</th>
                                    <th class="p-2.5">"Aksi"</th>
                                    <th class="p-2.5">"Entitas"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || logs.get()
                                    key=|l| l.id
                                    children=|l| view! {
                                        <tr class="border-b">
                                            <td class="p-2.5 text-muted-foreground whitespace-nowrap">{l.created_at.format("%Y-%m-%d %H:%M").to_string()}</td>
                                            <td class="p-2.5 font-medium">{l.username}</td>
                                            <td class="p-2.5"><Badge variant=BadgeVariant::Outline>{l.action}</Badge></td>
                                            <td class="p-2.5 text-muted-foreground">{l.entity_type}</td>
                                        </tr>
                                    }
                                />
                            </tbody>
                        </table>
                    </div>
                </Card>
            </Show>

            <Dialog open=form_open title="Form Pengguna Sobat K3">
                <form on:submit=save_user class="space-y-3">
                    <Show when=move || editing_id.get().is_none()>
                        <div class="space-y-1">
                            <Label>"Username"</Label>
                            <Input value=f_username placeholder="contoh: budi_k3" />
                        </div>
                    </Show>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1"><Label>"Nama Lengkap"</Label><Input value=f_name /></div>
                        <div class="space-y-1"><Label>"Email"</Label><Input value=f_email r#type="email" /></div>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1"><Label>"NIP"</Label><Input value=f_nip /></div>
                        <div class="space-y-1"><Label>"Jabatan P2K3"</Label><Input value=f_pos /></div>
                    </div>
                    <div class="space-y-1">
                        <Label>"Role"</Label>
                        <select
                            class="flex h-10 w-full rounded-md border border-input bg-card px-3 py-2 text-sm"
                            prop:value=move || f_role.get()
                            on:change=move |ev| f_role.set(event_target_value(&ev))
                        >
                            <option value="admin" selected=move || f_role.get() == "admin">"Admin"</option>
                            <option value="ketua_tim_k3" selected=move || f_role.get() == "ketua_tim_k3">"Ketua Tim K3"</option>
                            <option value="sekretaris_tim_k3" selected=move || f_role.get() == "sekretaris_tim_k3">"Sekretaris Tim K3"</option>
                            <option value="anggota_tim_k3" selected=move || f_role.get() == "anggota_tim_k3">"Anggota Tim K3"</option>
                        </select>
                    </div>
                    <Show when=move || editing_id.get().is_none()>
                        <div class="space-y-1">
                            <Label>"Password Awal (min. 6 karakter)"</Label>
                            <Input value=f_password r#type="password" />
                        </div>
                    </Show>
                    <Show when=move || editing_id.get().is_some()>
                        <label class="flex items-center gap-2 text-sm pt-1">
                            <input
                                type="checkbox"
                                prop:checked=move || f_active.get()
                                on:change=move |ev| f_active.set(event_target_checked(&ev))
                            />
                            <span>"Akun Aktif"</span>
                        </label>
                    </Show>
                    <div class="flex justify-end pt-2">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                            "Simpan Pengguna"
                        </Button>
                    </div>
                </form>
            </Dialog>

            <Dialog open=pw_open title="Reset Password Pengguna">
                <form on:submit=save_pw class="space-y-4">
                    <div class="space-y-1.5">
                        <Label>"Password Baru (minimal 6 karakter)"</Label>
                        <Input value=pw_new r#type="password" />
                    </div>
                    <div class="flex justify-end">
                        <Button r#type="submit">"Simpan Password Baru"</Button>
                    </div>
                </form>
            </Dialog>
        </div>
    }
}
