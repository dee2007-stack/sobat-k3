use crate::api::{api_list_lagging, api_update_lagging_meta, api_update_lagging_month};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::Dialog;
use crate::components::ui::icons::IconPencil;
use crate::components::ui::input::{Input, Label, Textarea};
use crate::state::AppState;
use leptos::prelude::*;
use shared::formulas::month_short_name;
use shared::models::{
    LaggingIndicatorDto, UpdateLaggingMetaRequest, UpdateLaggingMonthRequest,
};

#[component]
pub fn LaggingPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let items = RwSignal::new(Vec::<LaggingIndicatorDto>::new());
    let loading = RwSignal::new(true);

    // Input Bulanan Modal
    let month_open = RwSignal::new(false);
    let sel_item = RwSignal::new(Option::<LaggingIndicatorDto>::None);
    let sel_month = RwSignal::new(1i32);
    let f_val = RwSignal::new("0".to_string());
    let f_lost_days = RwSignal::new("0".to_string());
    let f_notes = RwSignal::new(String::new());

    // Meta / RCA Modal
    let meta_open = RwSignal::new(false);
    let f_rca = RwSignal::new(String::new());
    let f_capa = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let load = move || {
        let Some(token) = state.token.get() else {
            return;
        };
        let Some(period) = state.active_period.get() else {
            return;
        };
        loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_list_lagging(&period.id.to_string(), &token).await {
                Ok(list) => items.set(list),
                Err(e) => state.notify(e, true),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        load();
    });

    let open_month = move |ind: LaggingIndicatorDto, m: i32| {
        if ind.is_auto_calculated {
            state.notify("Indikator ini dihitung otomatis dari Tabel 1 & Insiden", false);
            return;
        }
        let md = ind.months.iter().find(|x| x.month == m).cloned();
        sel_item.set(Some(ind));
        sel_month.set(m);
        f_val.set(format!("{}", md.as_ref().map(|x| x.value).unwrap_or(0.0)));
        f_lost_days.set(format!("{}", md.as_ref().map(|x| x.lost_days).unwrap_or(0)));
        f_notes.set(md.map(|x| x.incident_notes).unwrap_or_default());
        month_open.set(true);
    };

    let open_meta = move |ind: LaggingIndicatorDto| {
        f_rca.set(ind.root_cause_analysis.clone());
        f_capa.set(ind.corrective_action.clone());
        sel_item.set(Some(ind));
        meta_open.set(true);
    };

    let save_month = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(ind) = sel_item.get_untracked() else {
            return;
        };
        let m = sel_month.get_untracked();
        let req = UpdateLaggingMonthRequest {
            value: f_val.get_untracked().parse().unwrap_or(0.0),
            lost_days: f_lost_days.get_untracked().parse().unwrap_or(0),
            incident_notes: f_notes.get_untracked(),
        };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_lagging_month(&ind.id.to_string(), m, &req, &token).await {
                Ok(_) => {
                    month_open.set(false);
                    state.notify("Data indikator tersimpan & statistik diperbarui", false);
                    load();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let save_meta = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(ind) = sel_item.get_untracked() else {
            return;
        };
        let req = UpdateLaggingMetaRequest {
            root_cause_analysis: f_rca.get_untracked(),
            corrective_action: f_capa.get_untracked(),
        };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_lagging_meta(&ind.id.to_string(), &req, &token).await {
                Ok(_) => {
                    meta_open.set(false);
                    state.notify("Analisis akar masalah disimpan", false);
                    load();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
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
            <div>
                <h1 class="text-xl md:text-2xl font-bold">"E. Pemantauan Indikator Tertinggal (Lagging Indicators)"</h1>
                <p class="text-xs md:text-sm text-muted-foreground">
                    "Statistik K3L Tahun 2027. Parameter 12–15 (Safe Manhours, FR, SR, TRIR) dihitung otomatis dari Tabel 1 dan data insiden."
                </p>
            </div>

            <Show when=move || loading.get()>
                <div class="h-40 rounded-xl bg-muted animate-pulse"></div>
            </Show>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                <For
                    each=move || items.get()
                    key=|i| i.id
                    children=move |ind| {
                        let ind_meta = StoredValue::new(ind.clone());
                        let ind_m = ind.clone();
                        let badge_v = if ind.compliance_status.contains("Tercapai") {
                            BadgeVariant::Success
                        } else if ind.compliance_status == "Terpantau" {
                            BadgeVariant::Secondary
                        } else {
                            BadgeVariant::Destructive
                        };
                        let val_fmt = if ind.indicator_key == "fr" || ind.indicator_key == "sr" || ind.indicator_key == "trir" {
                            format!("{:.2}", ind.annual_realization)
                        } else {
                            format!("{:.0}", ind.annual_realization)
                        };
                        view! {
                            <Card>
                                <CardHeader class="pb-2">
                                    <div class="flex items-start justify-between gap-2">
                                        <div>
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <Badge variant=BadgeVariant::Default>{format!("#{}", ind.item_number)}</Badge>
                                                <Badge variant=badge_v>{ind.compliance_status.clone()}</Badge>
                                                <Show when=move || ind.is_auto_calculated>
                                                    <Badge variant=BadgeVariant::Outline>"Otomatis"</Badge>
                                                </Show>
                                            </div>
                                            <CardTitle class="text-sm pt-1.5">{ind.parameter_name.clone()}</CardTitle>
                                        </div>
                                        <div class="text-right">
                                            <div class="text-lg font-bold text-primary tabular-nums">{val_fmt}</div>
                                            <div class="text-[10px] text-muted-foreground">{format!("Maks: {:.0}", ind.max_target)}</div>
                                        </div>
                                    </div>
                                </CardHeader>
                                <CardContent class="space-y-2.5 text-xs">
                                    <div class="text-muted-foreground">
                                        <span><strong>"Rumus/Ref: "</strong>{ind.reference_formula.clone()}</span>
                                    </div>
                                    <div class="grid grid-cols-6 sm:grid-cols-12 gap-1">
                                        <For
                                            each=move || ind.months.clone()
                                            key=|m| m.month
                                            children=move |m| {
                                                let ind_btn = ind_m.clone();
                                                let disp = if ind_btn.indicator_key == "fr" || ind_btn.indicator_key == "sr" || ind_btn.indicator_key == "trir" {
                                                    format!("{:.2}", m.value)
                                                } else {
                                                    format!("{:.0}", m.value)
                                                };
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="rounded border bg-muted/20 p-1 text-[10px] text-center hover:bg-accent cursor-pointer"
                                                        on:click=move |_| open_month(ind_btn.clone(), m.month)
                                                    >
                                                        <div class="text-muted-foreground">{month_short_name(m.month)}</div>
                                                        <div class="font-semibold tabular-nums">{disp}</div>
                                                    </button>
                                                }
                                            }
                                        />
                                    </div>
                                    <Show when=can_master>
                                        <div class="flex justify-end pt-2 border-t">
                                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(move |_| open_meta(ind_meta.get_value()))>
                                                <IconPencil class="w-3 h-3" />
                                                <span>"Akar Masalah & Koreksi"</span>
                                            </Button>
                                        </div>
                                    </Show>
                                </CardContent>
                            </Card>
                        }
                    }
                />
            </div>

            <Dialog open=month_open title="Input Kejadian / Statistik Bulanan">
                <form on:submit=save_month class="space-y-4">
                    <div class="text-xs text-muted-foreground">
                        {move || sel_item.get().map(|i| format!("{} ({})", i.parameter_name, month_short_name(sel_month.get()))).unwrap_or_default()}
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Jumlah Kejadian / Kasus"</Label>
                        <Input value=f_val r#type="number" />
                    </div>
                    <Show when=move || sel_item.get().map(|i| i.indicator_key == "lti").unwrap_or(false)>
                        <div class="space-y-1.5">
                            <Label>"Jumlah Hari Hilang (untuk perhitungan SR)"</Label>
                            <Input value=f_lost_days r#type="number" />
                        </div>
                    </Show>
                    <div class="space-y-1.5">
                        <Label>"Kronologi Singkat / Catatan"</Label>
                        <Textarea value=f_notes />
                    </div>
                    <div class="flex justify-end">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                            "Simpan"
                        </Button>
                    </div>
                </form>
            </Dialog>

            <Dialog open=meta_open title="Analisis Akar Masalah & Tindakan Korektif">
                <form on:submit=save_meta class="space-y-4">
                    <div class="space-y-1.5">
                        <Label>"Analisis Akar Masalah"</Label>
                        <Textarea value=f_rca rows=3 />
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Tindakan Korektif & Pencegahan"</Label>
                        <Textarea value=f_capa rows=3 />
                    </div>
                    <div class="flex justify-end">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                            "Simpan Analisis"
                        </Button>
                    </div>
                </form>
            </Dialog>
        </div>
    }
}
