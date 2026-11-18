use crate::api::{api_sign_period, api_update_period, download_xlsx};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::icons::{IconDownload, IconPrinter};
use crate::components::ui::input::{Input, Label, Textarea};
use crate::state::AppState;
use leptos::prelude::*;
use shared::models::{UpdatePeriodRequest, UserRole};

#[component]
pub fn ReportPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let f_doc = RwSignal::new(String::new());
    let f_rev = RwSignal::new(String::new());
    let f_sk = RwSignal::new(String::new());
    let f_legal = RwSignal::new(String::new());
    let f_date = RwSignal::new(String::new());
    let f_chair = RwSignal::new(String::new());
    let f_chair_nip = RwSignal::new(String::new());
    let f_sec = RwSignal::new(String::new());
    let f_sec_nip = RwSignal::new(String::new());
    let f_head = RwSignal::new(String::new());
    let f_head_nip = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(p) = state.active_period.get() {
            f_doc.set(p.doc_number);
            f_rev.set(p.revision);
            f_sk.set(p.sk_basis);
            f_legal.set(p.legal_basis);
            f_date.set(p.approval_date_text);
            f_chair.set(p.chair_name);
            f_chair_nip.set(p.chair_nip);
            f_sec.set(p.secretary_name);
            f_sec_nip.set(p.secretary_nip);
            f_head.set(p.head_name);
            f_head_nip.set(p.head_nip);
        }
    });

    let on_export_xlsx = Callback::new(move |_| {
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(p) = state.active_period.get_untracked() else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match download_xlsx(&p.id.to_string(), p.year, &token).await {
                Ok(_) => state.notify("File Excel (.xlsx) berhasil diunduh", false),
                Err(e) => state.notify(e, true),
            }
        });
    });

    let on_print = Callback::new(move |_| {
        if let Some(w) = web_sys::window() {
            let _ = w.print();
        }
    });

    let on_save_meta = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(p) = state.active_period.get_untracked() else {
            return;
        };
        let req = UpdatePeriodRequest {
            title: p.title,
            subtitle: p.subtitle,
            doc_number: f_doc.get_untracked(),
            revision: f_rev.get_untracked(),
            sk_basis: f_sk.get_untracked(),
            legal_basis: f_legal.get_untracked(),
            approval_date_text: f_date.get_untracked(),
            chair_name: f_chair.get_untracked(),
            chair_nip: f_chair_nip.get_untracked(),
            secretary_name: f_sec.get_untracked(),
            secretary_nip: f_sec_nip.get_untracked(),
            head_name: f_head.get_untracked(),
            head_nip: f_head_nip.get_untracked(),
        };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_period(&p.id.to_string(), &req, &token).await {
                Ok(updated) => {
                    state.active_period.set(Some(updated));
                    state.notify("Identitas dokumen & pejabat diperbarui", false);
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let sign_stage = move |stage: &'static str| {
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(p) = state.active_period.get_untracked() else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api_sign_period(&p.id.to_string(), stage, &token).await {
                Ok(updated) => {
                    state.active_period.set(Some(updated));
                    state.notify("Pengesahan berhasil dicatat", false);
                }
                Err(e) => state.notify(e, true),
            }
        });
    };

    let can_master = move || {
        state
            .user
            .get()
            .map(|u| u.role.can_manage_master())
            .unwrap_or(false)
    };

    view! {
        <div class="space-y-4">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 no-print">
                <div>
                    <h1 class="text-xl md:text-2xl font-bold">"Pengesahan & Ekspor Dokumen"</h1>
                    <p class="text-xs md:text-sm text-muted-foreground">
                        "Identitas dokumen terkendali, blok tanda tangan P2K3, serta ekspor Excel (.xlsx) & Cetak PDF."
                    </p>
                </div>
                <div class="flex gap-2">
                    <Button size=ButtonSize::Sm on_click=on_export_xlsx>
                        <IconDownload class="w-3.5 h-3.5" />
                        <span>"Download Excel (.xlsx)"</span>
                    </Button>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=on_print>
                        <IconPrinter class="w-3.5 h-3.5" />
                        <span>"Cetak / PDF"</span>
                    </Button>
                </div>
            </div>

            {move || state.active_period.get().map(|p| {
                let role = state.user.get().map(|u| u.role);
                let can_sec = matches!(role, Some(UserRole::SekretarisTimK3) | Some(UserRole::Admin));
                let can_chair = matches!(role, Some(UserRole::KetuaTimK3) | Some(UserRole::Admin));
                view! {
                    <Card>
                        <CardHeader class="bg-primary text-primary-foreground rounded-t-xl pb-5">
                            <CardTitle class="text-base md:text-lg">{p.title.clone()}</CardTitle>
                            <p class="text-xs text-emerald-100">{p.subtitle.clone()}</p>
                        </CardHeader>
                        <CardContent class="p-4 md:p-6 space-y-4 text-xs md:text-sm">
                            <div class="rounded-lg border-l-4 border-primary bg-muted/40 p-3 space-y-1">
                                <div><strong>"No. Dokumen: "</strong><code>{p.doc_number.clone()}</code> " | " <strong>"Revisi: "</strong><code>{p.revision.clone()}</code></div>
                                <div><strong>"Dasar SK: "</strong>{p.sk_basis.clone()}</div>
                                <div><strong>"Payung Hukum: "</strong>{p.legal_basis.clone()}</div>
                                <div class="pt-1">
                                    <Badge variant=BadgeVariant::Default>{format!("Status: {}", p.status.to_uppercase())}</Badge>
                                </div>
                            </div>

                            <div class="text-right text-xs pt-2">{p.approval_date_text.clone()}</div>
                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2 border-t">
                                <div class="rounded-lg border p-4 space-y-2">
                                    <div class="font-semibold text-xs">"Disiapkan Oleh,"</div>
                                    <div class="text-xs text-muted-foreground">"Ketua Tim P2K3 BBPOM di Bandung"</div>
                                    <div class="pt-6 font-semibold">{p.chair_name.clone()}</div>
                                    <div class="text-xs text-muted-foreground">{format!("NIP. {}", p.chair_nip)}</div>
                                    <Show when=move || p.chair_signed_at.is_some()>
                                        <Badge variant=BadgeVariant::Success>"Ditandatangani"</Badge>
                                    </Show>
                                    <Show when=move || can_chair>
                                        <div class="pt-2 no-print">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| sign_stage("chair"))>
                                                "Tanda Tangani (Ketua)"
                                            </Button>
                                        </div>
                                    </Show>
                                </div>

                                <div class="rounded-lg border p-4 space-y-2">
                                    <div class="font-semibold text-xs">"Diverifikasi Oleh,"</div>
                                    <div class="text-xs text-muted-foreground">"Sekretaris Tim P2K3 (Ahli K3)"</div>
                                    <div class="pt-6 font-semibold">{p.secretary_name.clone()}</div>
                                    <div class="text-xs text-muted-foreground">{format!("NIP. {}", p.secretary_nip)}</div>
                                    <Show when=move || p.secretary_verified_at.is_some()>
                                        <Badge variant=BadgeVariant::Success>"Terverifikasi"</Badge>
                                    </Show>
                                    <Show when=move || can_sec>
                                        <div class="pt-2 no-print">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| sign_stage("secretary"))>
                                                "Verifikasi (Sekretaris)"
                                            </Button>
                                        </div>
                                    </Show>
                                </div>

                                <div class="rounded-lg border p-4 space-y-2">
                                    <div class="font-semibold text-xs">"Disetujui & Diketahui Oleh,"</div>
                                    <div class="text-xs text-muted-foreground">"Kepala Balai Besar POM di Bandung"</div>
                                    <div class="pt-6 font-semibold">{p.head_name.clone()}</div>
                                    <div class="text-xs text-muted-foreground">{format!("NIP. {}", p.head_nip)}</div>
                                    <Show when=move || p.head_approved_at.is_some()>
                                        <Badge variant=BadgeVariant::Success>"Disetujui"</Badge>
                                    </Show>
                                    <Show when=move || can_chair>
                                        <div class="pt-2 no-print">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| sign_stage("head"))>
                                                "Catat Persetujuan"
                                            </Button>
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                }
            })}

            <Show when=can_master>
                <Card class="no-print">
                    <CardHeader class="border-b bg-muted/20">
                        <CardTitle>"Pengaturan Identitas Dokumen & Pejabat"</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <form on:submit=on_save_meta class="space-y-3">
                            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                                <div class="space-y-1"><Label>"No. Dokumen"</Label><Input value=f_doc /></div>
                                <div class="space-y-1"><Label>"Revisi"</Label><Input value=f_rev /></div>
                                <div class="space-y-1"><Label>"Tanggal Pengesahan"</Label><Input value=f_date /></div>
                            </div>
                            <div class="space-y-1"><Label>"Dasar SK"</Label><Input value=f_sk /></div>
                            <div class="space-y-1"><Label>"Payung Hukum"</Label><Textarea value=f_legal rows=2 /></div>
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <div class="space-y-1"><Label>"Nama Ketua P2K3"</Label><Input value=f_chair /></div>
                                <div class="space-y-1"><Label>"NIP Ketua P2K3"</Label><Input value=f_chair_nip /></div>
                                <div class="space-y-1"><Label>"Nama Sekretaris P2K3"</Label><Input value=f_sec /></div>
                                <div class="space-y-1"><Label>"NIP Sekretaris P2K3"</Label><Input value=f_sec_nip /></div>
                                <div class="space-y-1"><Label>"Nama Kepala Balai"</Label><Input value=f_head /></div>
                                <div class="space-y-1"><Label>"NIP Kepala Balai"</Label><Input value=f_head_nip /></div>
                            </div>
                            <div class="flex justify-end pt-2">
                                <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                                    "Simpan Pengaturan"
                                </Button>
                            </div>
                        </form>
                    </CardContent>
                </Card>
            </Show>
        </div>
    }
}
