use crate::api::{
    api_create_program, api_delete_program, api_get_dashboard, api_get_work_hours,
    api_list_lagging, api_list_programs, api_sign_period, api_update_lagging_meta,
    api_update_lagging_month, api_update_program, api_update_program_month, api_update_work_hour,
    download_xlsx,
};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent};
use crate::components::ui::dialog::Dialog;
use crate::components::ui::icons::{
    IconActivity, IconAlertTriangle, IconCheckCircle, IconClock, IconDownload, IconPencil,
    IconPlus, IconPrinter, IconTrash,
};
use crate::components::ui::input::{Input, Label, Textarea};
use crate::state::AppState;
use leptos::prelude::*;
use shared::formulas::month_short_name;
use shared::models::{
    DashboardSummaryDto, LaggingIndicatorDto, UpdateLaggingMetaRequest,
    UpdateLaggingMonthRequest, UpdateProgramMonthRequest, UpdateWorkHoursMonthRequest,
    UpsertWorkProgramRequest, UserRole, WorkHoursMonthDto, WorkHoursSummaryDto, WorkProgramDto,
    WorkProgramMonthDto,
};
use uuid::Uuid;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let summary = RwSignal::new(Option::<DashboardSummaryDto>::None);
    let work_hours = RwSignal::new(Option::<WorkHoursSummaryDto>::None);
    let programs = RwSignal::new(Vec::<WorkProgramDto>::new());
    let lagging = RwSignal::new(Vec::<LaggingIndicatorDto>::new());
    let loading = RwSignal::new(true);
    let saving = RwSignal::new(false);

    let reload_all = move || {
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(period) = state.active_period.get_untracked() else {
            return;
        };
        let pid = period.id.to_string();

        let (t1, p1) = (token.clone(), pid.clone());
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(d) = api_get_dashboard(&p1, &t1).await {
                summary.set(Some(d));
            }
            loading.set(false);
        });

        let (t2, p2) = (token.clone(), pid.clone());
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(wh) = api_get_work_hours(&p2, &t2).await {
                work_hours.set(Some(wh));
            }
        });

        let (t3, p3) = (token.clone(), pid.clone());
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(list) = api_list_programs(&p3, &t3).await {
                programs.set(list);
            }
        });

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(list) = api_list_lagging(&pid, &token).await {
                lagging.set(list);
            }
        });
    };

    Effect::new(move |_| {
        let _ = state.token.get();
        let _ = state.active_period.get();
        loading.set(true);
        reload_all();
    });

    // --- 1. Work Hours Modal ---
    let wh_open = RwSignal::new(false);
    let wh_month = RwSignal::new(1i32);
    let wh_days = RwSignal::new(String::new());
    let wh_hours = RwSignal::new(String::new());
    let wh_emp = RwSignal::new(String::new());

    let open_wh_edit = move |m: WorkHoursMonthDto| {
        wh_month.set(m.month);
        wh_days.set(format!("{}", m.effective_days));
        wh_hours.set(format!("{}", m.standard_hours_per_day));
        wh_emp.set(format!("{}", m.avg_employees_present));
        wh_open.set(true);
    };

    let save_wh = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(period) = state.active_period.get_untracked() else { return; };
        let req = UpdateWorkHoursMonthRequest {
            effective_days: wh_days.get_untracked().parse().unwrap_or(0.0),
            standard_hours_per_day: wh_hours.get_untracked().parse().unwrap_or(8.0),
            avg_employees_present: wh_emp.get_untracked().parse().unwrap_or(0.0),
        };
        let m = wh_month.get_untracked();
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_work_hour(&period.id.to_string(), m, &req, &token).await {
                Ok(_) => {
                    wh_open.set(false);
                    state.notify("Jam kerja diperbarui & Safe Manhours disinkronkan", false);
                    reload_all();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    // --- 2. Program Monthly Realization Modal ---
    let real_open = RwSignal::new(false);
    let real_prog = RwSignal::new(Option::<WorkProgramDto>::None);
    let real_month = RwSignal::new(1i32);
    let real_actual = RwSignal::new("0".to_string());
    let real_notes = RwSignal::new(String::new());

    let open_real = move |p: WorkProgramDto, m: i32| {
        let md = p.months.iter().find(|x| x.month == m).cloned();
        real_prog.set(Some(p));
        real_month.set(m);
        real_actual.set(format!("{}", md.as_ref().map(|x| x.actual_value).unwrap_or(0.0)));
        real_notes.set(md.map(|x| x.notes).unwrap_or_default());
        real_open.set(true);
    };

    let save_real = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(p) = real_prog.get_untracked() else { return; };
        let req = UpdateProgramMonthRequest {
            plan_label: None,
            plan_value: None,
            actual_value: real_actual.get_untracked().parse().unwrap_or(0.0),
            notes: real_notes.get_untracked(),
        };
        let m = real_month.get_untracked();
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_program_month(&p.id.to_string(), m, &req, &token).await {
                Ok(_) => {
                    real_open.set(false);
                    state.notify("Realisasi program disimpan", false);
                    reload_all();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    // --- 3. Program Master CRUD Modal ---
    let prog_open = RwSignal::new(false);
    let prog_edit_id = RwSignal::new(Option::<Uuid>::None);
    let f_pillar = RwSignal::new("A".to_string());
    let f_subcat = RwSignal::new(String::new());
    let f_item_no = RwSignal::new(String::new());
    let f_objective = RwSignal::new(String::new());
    let f_measurable = RwSignal::new(String::new());
    let f_code = RwSignal::new(String::new());
    let f_desc = RwSignal::new(String::new());
    let f_freq = RwSignal::new(String::new());
    let f_ref = RwSignal::new(String::new());
    let f_pic = RwSignal::new(String::new());
    let f_output = RwSignal::new(String::new());
    let f_target = RwSignal::new("1".to_string());
    let f_rca = RwSignal::new(String::new());
    let f_capa = RwSignal::new(String::new());

    let open_new_prog = Callback::new(move |_| {
        prog_edit_id.set(None);
        f_pillar.set("A".to_string());
        f_subcat.set(String::new());
        f_item_no.set(String::new());
        f_objective.set(String::new());
        f_measurable.set(String::new());
        f_code.set(String::new());
        f_desc.set(String::new());
        f_freq.set("1x per bulan".to_string());
        f_ref.set(String::new());
        f_pic.set(String::new());
        f_output.set(String::new());
        f_target.set("12".to_string());
        f_rca.set(String::new());
        f_capa.set(String::new());
        prog_open.set(true);
    });

    let open_edit_prog = move |p: WorkProgramDto| {
        prog_edit_id.set(Some(p.id));
        f_pillar.set(p.pillar);
        f_subcat.set(p.subcategory);
        f_item_no.set(p.item_number);
        f_objective.set(p.objective);
        f_measurable.set(p.measurable_target);
        f_code.set(p.code);
        f_desc.set(p.description);
        f_freq.set(p.frequency);
        f_ref.set(p.reference_law);
        f_pic.set(p.pic);
        f_output.set(p.output_evidence);
        f_target.set(format!("{}", p.annual_target));
        f_rca.set(p.root_cause_analysis);
        f_capa.set(p.corrective_preventive_action);
        prog_open.set(true);
    };

    let save_prog = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(period) = state.active_period.get_untracked() else { return; };
        let pillar = f_pillar.get_untracked();
        let pillar_title = match pillar.as_str() {
            "A" => "A. KESELAMATAN KERJA (SAFETY) — MEMENUHI 5 UNSUR WAJIB PROGRAM P2K3 (PASAL 10 PERMENAKER 13/2025)".to_string(),
            "B" => "B. KESEHATAN KERJA, HIGIENE INDUSTRI & SANITAIR (HEALTH)".to_string(),
            "C" => "C. LINGKUNGAN HIDUP & PENGELOLAAN LIMBAH B3 (ENVIRONMENT)".to_string(),
            _ => "D. MUTU LAYANAN K3, SARANA PRASARANA & LINGKUNGAN KERJA INKLUSIF".to_string(),
        };
        let existing_months = prog_edit_id
            .get_untracked()
            .and_then(|eid| programs.get_untracked().into_iter().find(|p| p.id == eid))
            .map(|p| p.months)
            .unwrap_or_else(|| {
                (1..=12)
                    .map(|m| WorkProgramMonthDto {
                        month: m,
                        plan_label: "Rencana".to_string(),
                        plan_value: 1.0,
                        actual_value: 0.0,
                        notes: String::new(),
                    })
                    .collect()
            });
        let req = UpsertWorkProgramRequest {
            pillar,
            pillar_title,
            subcategory: f_subcat.get_untracked(),
            item_number: f_item_no.get_untracked(),
            objective: f_objective.get_untracked(),
            measurable_target: f_measurable.get_untracked(),
            code: f_code.get_untracked(),
            description: f_desc.get_untracked(),
            frequency: f_freq.get_untracked(),
            reference_law: f_ref.get_untracked(),
            pic: f_pic.get_untracked(),
            output_evidence: f_output.get_untracked(),
            annual_target: f_target.get_untracked().parse().unwrap_or(1.0),
            is_target_auto_sum: false,
            root_cause_analysis: f_rca.get_untracked(),
            corrective_preventive_action: f_capa.get_untracked(),
            sort_order: 99,
            months: existing_months,
        };
        let eid = prog_edit_id.get_untracked();
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let res = match eid {
                Some(id) => api_update_program(&id.to_string(), &req, &token).await,
                None => api_create_program(&period.id.to_string(), &req, &token).await,
            };
            match res {
                Ok(_) => {
                    prog_open.set(false);
                    state.notify("Program kerja berhasil disimpan", false);
                    reload_all();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let delete_prog = Callback::new(move |_| {
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(id) = prog_edit_id.get_untracked() else { return; };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete_program(&id.to_string(), &token).await {
                Ok(_) => {
                    prog_open.set(false);
                    state.notify("Program kerja dihapus", false);
                    reload_all();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    });

    // --- 4. Lagging Monthly & Meta Modals ---
    let lag_m_open = RwSignal::new(false);
    let lag_sel = RwSignal::new(Option::<LaggingIndicatorDto>::None);
    let lag_month = RwSignal::new(1i32);
    let lag_val = RwSignal::new("0".to_string());
    let lag_lost = RwSignal::new("0".to_string());
    let lag_notes = RwSignal::new(String::new());

    let lag_meta_open = RwSignal::new(false);
    let lag_rca = RwSignal::new(String::new());
    let lag_capa = RwSignal::new(String::new());

    let open_lag_month = move |ind: LaggingIndicatorDto, m: i32| {
        if ind.is_auto_calculated {
            state.notify("Indikator ini dihitung otomatis dari Tabel 1 & Insiden", false);
            return;
        }
        let md = ind.months.iter().find(|x| x.month == m).cloned();
        lag_sel.set(Some(ind));
        lag_month.set(m);
        lag_val.set(format!("{}", md.as_ref().map(|x| x.value).unwrap_or(0.0)));
        lag_lost.set(format!("{}", md.as_ref().map(|x| x.lost_days).unwrap_or(0)));
        lag_notes.set(md.map(|x| x.incident_notes).unwrap_or_default());
        lag_m_open.set(true);
    };

    let open_lag_meta = move |ind: LaggingIndicatorDto| {
        lag_rca.set(ind.root_cause_analysis.clone());
        lag_capa.set(ind.corrective_action.clone());
        lag_sel.set(Some(ind));
        lag_meta_open.set(true);
    };

    let save_lag_month = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(ind) = lag_sel.get_untracked() else { return; };
        let req = UpdateLaggingMonthRequest {
            value: lag_val.get_untracked().parse().unwrap_or(0.0),
            lost_days: lag_lost.get_untracked().parse().unwrap_or(0),
            incident_notes: lag_notes.get_untracked(),
        };
        let m = lag_month.get_untracked();
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_lagging_month(&ind.id.to_string(), m, &req, &token).await {
                Ok(_) => {
                    lag_m_open.set(false);
                    state.notify("Data insiden disimpan & statistik diperbarui", false);
                    reload_all();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let save_lag_meta = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(ind) = lag_sel.get_untracked() else { return; };
        let req = UpdateLaggingMetaRequest {
            root_cause_analysis: lag_rca.get_untracked(),
            corrective_action: lag_capa.get_untracked(),
        };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_lagging_meta(&ind.id.to_string(), &req, &token).await {
                Ok(_) => {
                    lag_meta_open.set(false);
                    state.notify("Analisis akar masalah disimpan", false);
                    reload_all();
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    // --- 5. Sign Period ---
    let sign_stage = move |stage: &'static str| {
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(p) = state.active_period.get_untracked() else { return; };
        wasm_bindgen_futures::spawn_local(async move {
            match api_sign_period(&p.id.to_string(), stage, &token).await {
                Ok(updated) => {
                    state.active_period.set(Some(updated));
                    state.notify("Pengesahan dokumen berhasil dicatat", false);
                }
                Err(e) => state.notify(e, true),
            }
        });
    };

    let on_export_xlsx = Callback::new(move |_| {
        let Some(token) = state.token.get_untracked() else { return; };
        let Some(p) = state.active_period.get_untracked() else { return; };
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

    let can_master = move || {
        state
            .user
            .get()
            .map(|u| u.role.can_manage_master())
            .unwrap_or(false)
    };

    view! {
        <div class="space-y-6">
            // Document Header Banner
            {move || state.active_period.get().map(|p| view! {
                <div class="rounded-xl bg-primary text-primary-foreground p-5 sm:p-6 shadow-sm space-y-3">
                    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
                        <div class="space-y-1">
                            <h1 class="text-base sm:text-lg md:text-xl font-bold tracking-tight leading-snug">
                                {p.title}
                            </h1>
                            <p class="text-xs text-emerald-100 leading-relaxed">
                                {p.subtitle}
                            </p>
                        </div>
                        <div class="flex flex-wrap gap-2 shrink-0 no-print">
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=on_export_xlsx>
                                <IconDownload class="w-3.5 h-3.5" />
                                <span>"Excel (.xlsx)"</span>
                            </Button>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm class="bg-white/10 border-white/25 text-white hover:bg-white/20 hover:text-white" on_click=on_print>
                                <IconPrinter class="w-3.5 h-3.5" />
                                <span>"Cetak / PDF"</span>
                            </Button>
                        </div>
                    </div>
                </div>
            })}

            // KPI Summary Strip
            {move || summary.get().map(|d| view! {
                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 no-print">
                    <Card>
                        <CardContent class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs font-medium text-muted-foreground">
                                <span>"Safe Manhours"</span>
                                <IconClock class="w-4 h-4 text-primary" />
                            </div>
                            <div class="text-2xl font-bold text-primary tabular-nums">{format!("{:.0}", d.safe_manhours)}</div>
                            <div class="text-[11px] text-muted-foreground">{format!("{:.0} hari kerja efektif", d.total_effective_days)}</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs font-medium text-muted-foreground">
                                <span>"Capaian Program"</span>
                                <IconCheckCircle class="w-4 h-4 text-emerald-600" />
                            </div>
                            <div class="text-2xl font-bold text-emerald-700 tabular-nums">{format!("{:.1}%", d.overall_achievement_pct)}</div>
                            <div class="text-[11px] text-muted-foreground">"Rata-rata Pilar A–D"</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs font-medium text-muted-foreground">
                                <span>"FR / SR"</span>
                                <IconActivity class="w-4 h-4 text-emerald-600" />
                            </div>
                            <div class="text-2xl font-bold tabular-nums">{format!("{:.2} / {:.2}", d.fr, d.sr)}</div>
                            <div class="text-[11px] text-muted-foreground">{format!("Fatality: {:.0} | LTI: {:.0}", d.total_fatalities, d.total_lti)}</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs font-medium text-muted-foreground">
                                <span>"TRIR & Nearmiss"</span>
                                <IconAlertTriangle class="w-4 h-4 text-amber-600" />
                            </div>
                            <div class="text-2xl font-bold tabular-nums">{format!("{:.2}", d.trir)}</div>
                            <div class="text-[11px] text-muted-foreground">{format!("Nearmiss tercatat: {:.0}", d.total_nearmiss)}</div>
                        </CardContent>
                    </Card>
                </div>
            })}

            <Show when=move || loading.get()>
                <div class="h-48 rounded-xl bg-muted animate-pulse"></div>
            </Show>

            // SECTION 1: Jam Kerja Efektif (Interactive)
            <Card>
                <CardContent class="space-y-4">
                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 border-b-2 border-border pb-2">
                        <h2 class="text-sm md:text-base font-bold text-primary">
                            "1. Perhitungan Kumulatif Jam Kerja Efektif & Jam Kerja Aman ASN Tahun 2027"
                        </h2>
                        <span class="text-[11px] text-muted-foreground no-print">"Klik bulan untuk mengisi kehadiran pegawai"</span>
                    </div>
                    <div class="text-xs text-muted-foreground bg-emerald-50/70 border-l-4 border-primary px-4 py-3 rounded-r-lg leading-relaxed">
                        <strong>"Petunjuk Pengisian: "</strong>
                        "Hari kerja efektif ASN tahun 2027 telah disesuaikan dengan SKB 3 Menteri ("
                        <strong>"242 hari kerja efektif"</strong>
                        "). Klik kolom bulan atau angka pada baris "
                        <em>"Rata-rata jumlah pegawai hadir per hari"</em>
                        " untuk langsung mengubah data; indikator "
                        <em>"Safe Manhours"</em>
                        " akan terhitung otomatis."
                    </div>
                    {move || work_hours.get().map(|wh| {
                        let months_sv = StoredValue::new(wh.months.clone());
                        view! {
                            <div class="overflow-x-auto rounded-lg border">
                                <table class="w-full text-[11.5px] border-collapse">
                                    <thead>
                                        <tr class="bg-primary text-primary-foreground">
                                            <th class="p-2 text-left border border-emerald-800 min-w-[240px]">"Parameter Perhitungan"</th>
                                            <For
                                                each=move || months_sv.get_value()
                                                key=|m| m.month
                                                children=move |m| {
                                                    let m_sv = StoredValue::new(m.clone());
                                                    view! {
                                                        <th class="p-2 text-center border border-emerald-800 min-w-[50px]">
                                                            <div>{month_short_name(m.month)}</div>
                                                            <Show when=can_master>
                                                                <button
                                                                    type="button"
                                                                    class="mt-0.5 inline-flex items-center gap-0.5 text-[10px] underline text-emerald-100 hover:text-white cursor-pointer no-print"
                                                                    on:click=move |_| open_wh_edit(m_sv.get_value())
                                                                >
                                                                    <IconPencil class="w-2.5 h-2.5" />
                                                                    <span>"Isi"</span>
                                                                </button>
                                                            </Show>
                                                        </th>
                                                    }
                                                }
                                            />
                                            <th class="p-2 text-center border border-emerald-800 min-w-[95px]">"Total / Kumulatif 2027"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr class="border-b">
                                            <td class="p-2 border font-medium">"Hari kerja efektif ASN per bulan (hari)"</td>
                                            <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2 border text-right tabular-nums">{format!("{:.0}", m.effective_days)}</td> } />
                                            <td class="p-2 border text-right font-bold tabular-nums">{format!("{:.0}", wh.total_effective_days)}</td>
                                        </tr>
                                        <tr class="border-b">
                                            <td class="p-2 border font-medium">"Standar jam kerja efektif per hari (jam)"</td>
                                            <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2 border text-right tabular-nums">{format!("{:.0}", m.standard_hours_per_day)}</td> } />
                                            <td class="p-2 border text-right tabular-nums">{format!("{:.1}", wh.avg_standard_hours)}</td>
                                        </tr>
                                        <tr class="border-b">
                                            <td class="p-2 border font-medium">"Rata-rata jumlah pegawai hadir per hari (orang)"</td>
                                            <For
                                                each=move || months_sv.get_value()
                                                key=|m| m.month
                                                children=move |m| {
                                                    let m_sv = StoredValue::new(m.clone());
                                                    view! {
                                                        <td
                                                            class="p-2 border text-right tabular-nums font-semibold text-primary hover:bg-emerald-50 cursor-pointer"
                                                            on:click=move |_| {
                                                                if can_master() {
                                                                    open_wh_edit(m_sv.get_value());
                                                                }
                                                            }
                                                        >
                                                            {format!("{:.0}", m.avg_employees_present)}
                                                        </td>
                                                    }
                                                }
                                            />
                                            <td class="p-2 border text-right tabular-nums">{format!("{:.1}", wh.avg_employees_present)}</td>
                                        </tr>
                                        <tr class="border-b bg-emerald-50/60 font-semibold">
                                            <td class="p-2 border">"Total jam kerja seluruh pegawai per hari (jam)"</td>
                                            <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2 border text-right tabular-nums">{format!("{:.0}", m.total_daily_hours)}</td> } />
                                            <td class="p-2 border text-right tabular-nums">{format!("{:.0}", wh.sum_daily_hours)}</td>
                                        </tr>
                                        <tr class="bg-emerald-100/70 font-bold text-emerald-950">
                                            <td class="p-2 border">"Total jam kerja kumulatif per bulan (jam)"</td>
                                            <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2 border text-right tabular-nums">{format!("{:.0}", m.cumulative_monthly_hours)}</td> } />
                                            <td class="p-2 border text-right tabular-nums">{format!("{:.0}", wh.total_cumulative_hours)}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        }
                    })}
                </CardContent>
            </Card>

            // SECTION 2: Matriks TSP + Bagian E + Pengesahan (Interactive)
            <Card>
                <CardContent class="space-y-4">
                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 border-b-2 border-border pb-2">
                        <h2 class="text-sm md:text-base font-bold text-primary">
                            "2. Matriks Tujuan, Sasaran & Program Kerja (TSP) K3L / P2K3 BBPOM di Bandung Tahun 2027"
                        </h2>
                        <Show when=can_master>
                            <Button size=ButtonSize::Sm class="no-print shrink-0" on_click=open_new_prog>
                                <IconPlus class="w-3.5 h-3.5" />
                                <span>"Tambah Program"</span>
                            </Button>
                        </Show>
                    </div>

                    {move || state.active_period.get().map(|p| view! {
                        <div class="text-xs text-muted-foreground bg-emerald-50/70 border-l-4 border-primary px-4 py-3 rounded-r-lg leading-relaxed">
                            <strong>"Identitas Dokumen Terkendali: "</strong>
                            "No. Dokumen: " <code class="px-1.5 py-0.5 rounded bg-white border">{p.doc_number}</code>
                            " | Revisi: " <code class="px-1.5 py-0.5 rounded bg-white border">{p.revision}</code>
                            " | Dasar Pembentukan Tim: " <strong>{p.sk_basis}</strong>
                            " | Payung Hukum: " {p.legal_basis}
                        </div>
                    })}

                    <div class="text-[11px] text-muted-foreground no-print">
                        "Tip: Klik kode program (mis. A.1.1) untuk Edit/Hapus Master, atau klik sel bulan (Jan–Des) untuk input realisasi / insiden."
                    </div>

                    <div class="overflow-x-auto rounded-lg border">
                        <table class="w-full text-[11px] border-collapse">
                            <thead>
                                <tr class="bg-primary text-primary-foreground text-center">
                                    <th rowspan="2" class="p-2 border border-emerald-800">"No."</th>
                                    <th colspan="2" class="p-2 border border-emerald-800">"INDIKATOR TERTINGGAL (HASIL / LAGGING)"</th>
                                    <th colspan="4" class="p-2 border border-emerald-800">"INDIKATOR TERDEPAN (PROGRAM KERJA / LEADING)"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800 min-w-[140px]">"Penanggung Jawab (PIC)"</th>
                                    <th colspan="12" class="p-2 border border-emerald-800">"Jadwal Pelaksanaan Tahun 2027"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800 min-w-[140px]">"Output / Bukti Dukung"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800">"Target Tahunan"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800">"Realisasi Tahunan"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800">"Capaian (%)"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800 min-w-[120px]">"Analisis Akar Masalah (<100%)"</th>
                                    <th rowspan="2" class="p-2 border border-emerald-800 min-w-[130px]">"Tindakan Perbaikan & Pencegahan"</th>
                                </tr>
                                <tr class="bg-primary text-primary-foreground text-center">
                                    <th class="p-1.5 border border-emerald-800 min-w-[130px]">"Tujuan"</th>
                                    <th class="p-1.5 border border-emerald-800 min-w-[150px]">"Sasaran Terukur"</th>
                                    <th class="p-1.5 border border-emerald-800">"Kode"</th>
                                    <th class="p-1.5 border border-emerald-800 min-w-[220px]">"Uraian Program Kerja"</th>
                                    <th class="p-1.5 border border-emerald-800 min-w-[100px]">"Frekuensi"</th>
                                    <th class="p-1.5 border border-emerald-800 min-w-[130px]">"Referensi / Dasar Hukum"</th>
                                    {(1..=12).map(|m| view! {
                                        <th class="p-1.5 border border-emerald-800 min-w-[44px]">{month_short_name(m)}</th>
                                    }).collect_view()}
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let list = programs.get();
                                    let mut last_pillar = String::new();
                                    let mut last_subcat = String::new();
                                    let mut rows_view: Vec<AnyView> = Vec::new();

                                    for p in list {
                                        if p.pillar != last_pillar {
                                            last_pillar = p.pillar.clone();
                                            last_subcat.clear();
                                            rows_view.push(view! {
                                                <tr class="bg-emerald-100 text-emerald-950 font-bold text-xs">
                                                    <td colspan="26" class="p-2 border">{p.pillar_title.clone()}</td>
                                                </tr>
                                            }.into_any());
                                        }
                                        if !p.subcategory.is_empty() && p.subcategory != last_subcat {
                                            last_subcat = p.subcategory.clone();
                                            rows_view.push(view! {
                                                <tr class="bg-slate-100 text-slate-900 font-bold">
                                                    <td colspan="26" class="p-2 border">{p.subcategory.clone()}</td>
                                                </tr>
                                            }.into_any());
                                        }
                                        let months_sv = StoredValue::new(p.months.clone());
                                        let p_edit_sv = StoredValue::new(p.clone());
                                        let p_real_sv = StoredValue::new(p.clone());
                                        rows_view.push(view! {
                                            <tr class="hover:bg-muted/30">
                                                <td class="p-2 border text-center">{p.item_number.clone()}</td>
                                                <td class="p-2 border">{p.objective.clone()}</td>
                                                <td class="p-2 border whitespace-pre-line">{p.measurable_target.clone()}</td>
                                                <td class="p-2 border text-center">
                                                    <button
                                                        type="button"
                                                        class="font-semibold text-primary hover:underline cursor-pointer"
                                                        on:click=move |_| {
                                                            if can_master() {
                                                                open_edit_prog(p_edit_sv.get_value());
                                                            }
                                                        }
                                                    >
                                                        {p.code.clone()}
                                                    </button>
                                                </td>
                                                <td class="p-2 border">{p.description.clone()}</td>
                                                <td class="p-2 border">{p.frequency.clone()}</td>
                                                <td class="p-2 border">{p.reference_law.clone()}</td>
                                                <td class="p-2 border">{p.pic.clone()}</td>
                                                <For
                                                    each=move || months_sv.get_value()
                                                    key=|m| m.month
                                                    children=move |m| {
                                                        let cls = if m.actual_value > 0.0 {
                                                            "bg-emerald-100 text-emerald-900 font-semibold"
                                                        } else if m.plan_label == "Tentatif" {
                                                            "bg-amber-100 text-amber-900 font-semibold"
                                                        } else if !m.plan_label.is_empty() {
                                                            "bg-emerald-50 text-emerald-800 font-semibold"
                                                        } else {
                                                            ""
                                                        };
                                                        view! {
                                                            <td
                                                                class=format!("p-1.5 border text-center cursor-pointer hover:ring-1 hover:ring-primary {}", cls)
                                                                on:click=move |_| open_real(p_real_sv.get_value(), m.month)
                                                            >
                                                                {if m.actual_value > 0.0 {
                                                                    format!("{} ({:.0})", m.plan_label, m.actual_value)
                                                                } else {
                                                                    m.plan_label.clone()
                                                                }}
                                                            </td>
                                                        }
                                                    }
                                                />
                                                <td class="p-2 border">{p.output_evidence.clone()}</td>
                                                <td class="p-2 border text-right tabular-nums">{format!("{:.0}", p.annual_target)}</td>
                                                <td class="p-2 border text-right tabular-nums">{format!("{:.0}", p.annual_realization)}</td>
                                                <td class="p-2 border text-right tabular-nums font-semibold">{format!("{:.0}%", p.achievement_pct)}</td>
                                                <td class="p-2 border">{p.root_cause_analysis.clone()}</td>
                                                <td class="p-2 border">{p.corrective_preventive_action.clone()}</td>
                                            </tr>
                                        }.into_any());
                                    }
                                    rows_view
                                }}

                                // BAGIAN E: LAGGING INDICATORS
                                <tr class="bg-emerald-100 text-emerald-950 font-bold text-xs">
                                    <td colspan="26" class="p-2 border">"E. PEMANTAUAN INDIKATOR TERTINGGAL (LAGGING INDICATORS) & STATISTIK K3L TAHUN 2027"</td>
                                </tr>
                                <tr class="bg-primary text-primary-foreground text-center font-semibold">
                                    <th class="p-2 border border-emerald-800">"No."</th>
                                    <th colspan="4" class="p-2 border border-emerald-800">"Parameter Indikator Tertinggal (Lagging Indicator)"</th>
                                    <th class="p-2 border border-emerald-800">"Frekuensi Pantau"</th>
                                    <th class="p-2 border border-emerald-800">"Referensi / Rumus Perhitungan"</th>
                                    <th class="p-2 border border-emerald-800">"Penanggung Jawab"</th>
                                    {(1..=12).map(|m| view! {
                                        <th class="p-1.5 border border-emerald-800">{month_short_name(m)}</th>
                                    }).collect_view()}
                                    <th class="p-2 border border-emerald-800">"Keterangan"</th>
                                    <th class="p-2 border border-emerald-800">"Target Maks."</th>
                                    <th class="p-2 border border-emerald-800">"Realisasi"</th>
                                    <th class="p-2 border border-emerald-800">"Status Kepatuhan"</th>
                                    <th class="p-2 border border-emerald-800">"Analisis Akar Masalah"</th>
                                    <th class="p-2 border border-emerald-800">"Tindakan Korektif"</th>
                                </tr>
                                <For
                                    each=move || lagging.get()
                                    key=|l| l.id
                                    children=move |ind| {
                                        let is_rate = ind.indicator_key == "fr" || ind.indicator_key == "sr" || ind.indicator_key == "trir";
                                        let months_sv = StoredValue::new(ind.months.clone());
                                        let ind_m_sv = StoredValue::new(ind.clone());
                                        let ind_meta_sv = StoredValue::new(ind.clone());
                                        let status_v = if ind.compliance_status.contains("Tercapai") {
                                            BadgeVariant::Success
                                        } else if ind.compliance_status == "Terpantau" {
                                            BadgeVariant::Secondary
                                        } else {
                                            BadgeVariant::Destructive
                                        };
                                        view! {
                                            <tr class="hover:bg-muted/30">
                                                <td class="p-2 border text-center">{ind.item_number}</td>
                                                <td colspan="4" class="p-2 border font-medium">{ind.parameter_name.clone()}</td>
                                                <td class="p-2 border">{ind.frequency.clone()}</td>
                                                <td class="p-2 border">{ind.reference_formula.clone()}</td>
                                                <td class="p-2 border">{ind.pic.clone()}</td>
                                                <For
                                                    each=move || months_sv.get_value()
                                                    key=|m| m.month
                                                    children=move |m| view! {
                                                        <td
                                                            class="p-1.5 border text-right tabular-nums cursor-pointer hover:bg-emerald-50"
                                                            on:click=move |_| open_lag_month(ind_m_sv.get_value(), m.month)
                                                        >
                                                            {if is_rate { format!("{:.2}", m.value) } else { format!("{:.0}", m.value) }}
                                                        </td>
                                                    }
                                                />
                                                <td class="p-2 border">{ind.notes.clone()}</td>
                                                <td class="p-2 border text-right tabular-nums">{if is_rate { format!("{:.2}", ind.max_target) } else { format!("{:.0}", ind.max_target) }}</td>
                                                <td class="p-2 border text-right tabular-nums font-semibold">{if is_rate { format!("{:.2}", ind.annual_realization) } else { format!("{:.0}", ind.annual_realization) }}</td>
                                                <td class="p-2 border text-center">
                                                    <Badge variant=status_v>{ind.compliance_status.clone()}</Badge>
                                                </td>
                                                <td
                                                    class="p-2 border cursor-pointer hover:bg-emerald-50"
                                                    on:click=move |_| {
                                                        if can_master() {
                                                            open_lag_meta(ind_meta_sv.get_value());
                                                        }
                                                    }
                                                >
                                                    {if ind.root_cause_analysis.is_empty() && can_master() { "Klik untuk isi".to_string() } else { ind.root_cause_analysis.clone() }}
                                                </td>
                                                <td
                                                    class="p-2 border cursor-pointer hover:bg-emerald-50"
                                                    on:click=move |_| {
                                                        if can_master() {
                                                            open_lag_meta(ind_meta_sv.get_value());
                                                        }
                                                    }
                                                >
                                                    {ind.corrective_action.clone()}
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </div>

                    // BLOK PENGESAHAN (Interactive)
                    {move || state.active_period.get().map(|p| {
                        let role = state.user.get().map(|u| u.role);
                        let can_sec = matches!(role, Some(UserRole::SekretarisTimK3) | Some(UserRole::Admin));
                        let can_chair = matches!(role, Some(UserRole::KetuaTimK3) | Some(UserRole::Admin));
                        view! {
                        <div class="pt-4 space-y-4">
                            <div class="text-right text-xs">{p.approval_date_text}</div>
                            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 text-xs pt-2">
                                <div class="space-y-1.5">
                                    <div class="font-bold">"Disiapkan Oleh,"</div>
                                    <div>"Ketua Tim P2K3 BBPOM di Bandung"</div>
                                    <div class="pt-10 border-b border-foreground/60 w-56 font-semibold">{p.chair_name}</div>
                                    <div>{format!("NIP. {}", p.chair_nip)}</div>
                                    <Show when=move || p.chair_signed_at.is_some()>
                                        <Badge variant=BadgeVariant::Success>"Ditandatangani"</Badge>
                                    </Show>
                                    <Show when=move || can_chair>
                                        <div class="pt-1 no-print">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| sign_stage("chair"))>
                                                "Tanda Tangani"
                                            </Button>
                                        </div>
                                    </Show>
                                </div>
                                <div class="space-y-1.5">
                                    <div class="font-bold">"Diverifikasi Oleh,"</div>
                                    <div>"Sekretaris Tim P2K3 (Ahli K3)"</div>
                                    <div class="pt-10 border-b border-foreground/60 w-56 font-semibold">{p.secretary_name}</div>
                                    <div>{format!("NIP. {}", p.secretary_nip)}</div>
                                    <Show when=move || p.secretary_verified_at.is_some()>
                                        <Badge variant=BadgeVariant::Success>"Terverifikasi"</Badge>
                                    </Show>
                                    <Show when=move || can_sec>
                                        <div class="pt-1 no-print">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| sign_stage("secretary"))>
                                                "Verifikasi"
                                            </Button>
                                        </div>
                                    </Show>
                                </div>
                                <div class="space-y-1.5">
                                    <div class="font-bold">"Disetujui & Diketahui Oleh,"</div>
                                    <div>"Kepala Balai Besar POM di Bandung"</div>
                                    <div class="pt-10 border-b border-foreground/60 w-56 font-semibold">{p.head_name}</div>
                                    <div>{format!("NIP. {}", p.head_nip)}</div>
                                    <Show when=move || p.head_approved_at.is_some()>
                                        <Badge variant=BadgeVariant::Success>"Disetujui"</Badge>
                                    </Show>
                                    <Show when=move || can_chair>
                                        <div class="pt-1 no-print">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| sign_stage("head"))>
                                                "Catat Persetujuan"
                                            </Button>
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        </div>
                        }
                    })}
                </CardContent>
            </Card>

            // MODAL 1: Edit Jam Kerja Bulanan
            <Dialog open=wh_open title="Edit Jam Kerja Bulanan">
                <form on:submit=save_wh class="space-y-4">
                    <div class="text-sm font-semibold text-primary">
                        {move || format!("Bulan: {} 2027", month_short_name(wh_month.get()))}
                    </div>
                    <div class="space-y-1.5"><Label>"Hari Kerja Efektif ASN (hari)"</Label><Input value=wh_days r#type="number" /></div>
                    <div class="space-y-1.5"><Label>"Standar Jam Kerja per Hari (jam)"</Label><Input value=wh_hours r#type="number" /></div>
                    <div class="space-y-1.5"><Label>"Rata-rata Jumlah Pegawai Hadir per Hari (orang)"</Label><Input value=wh_emp r#type="number" /></div>
                    <div class="flex justify-end pt-2">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>"Simpan & Hitung Ulang"</Button>
                    </div>
                </form>
            </Dialog>

            // MODAL 2: Input Realisasi Program Bulanan
            <Dialog open=real_open title="Input Realisasi Program Bulanan">
                <form on:submit=save_real class="space-y-4">
                    <div class="text-xs text-muted-foreground">
                        {move || real_prog.get().map(|p| format!("{} — {} ({})", p.code, p.description, month_short_name(real_month.get()))).unwrap_or_default()}
                    </div>
                    <div class="space-y-1.5"><Label>"Nilai Realisasi Bulan Ini"</Label><Input value=real_actual r#type="number" /></div>
                    <div class="space-y-1.5"><Label>"Catatan / Link Bukti Dukung"</Label><Textarea value=real_notes /></div>
                    <div class="flex justify-end">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>"Simpan Realisasi"</Button>
                    </div>
                </form>
            </Dialog>

            // MODAL 3: Master CRUD Program Kerja
            <Dialog open=prog_open title="Form Master Program Kerja K3L" max_width="max-w-2xl">
                <form on:submit=save_prog class="space-y-3">
                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                        <div class="space-y-1"><Label>"Pilar (A/B/C/D)"</Label><Input value=f_pillar placeholder="A" /></div>
                        <div class="space-y-1"><Label>"Kode Program"</Label><Input value=f_code placeholder="A.2.16" /></div>
                        <div class="space-y-1"><Label>"Target Tahunan"</Label><Input value=f_target r#type="number" /></div>
                    </div>
                    <div class="space-y-1"><Label>"Sub-kategori (opsional)"</Label><Input value=f_subcat /></div>
                    <div class="space-y-1"><Label>"Tujuan"</Label><Input value=f_objective /></div>
                    <div class="space-y-1"><Label>"Sasaran Terukur"</Label><Textarea value=f_measurable rows=2 /></div>
                    <div class="space-y-1"><Label>"Uraian Program Kerja"</Label><Textarea value=f_desc rows=3 /></div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1"><Label>"Frekuensi Pelaksanaan"</Label><Input value=f_freq /></div>
                        <div class="space-y-1"><Label>"Penanggung Jawab (PIC)"</Label><Input value=f_pic /></div>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1"><Label>"Referensi / Dasar Hukum"</Label><Input value=f_ref /></div>
                        <div class="space-y-1"><Label>"Output / Bukti Dukung"</Label><Input value=f_output /></div>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1"><Label>"Analisis Akar Masalah (<100%)"</Label><Textarea value=f_rca rows=2 /></div>
                        <div class="space-y-1"><Label>"Tindakan Perbaikan & Pencegahan"</Label><Textarea value=f_capa rows=2 /></div>
                    </div>
                    <div class="flex justify-between gap-2 pt-2">
                        <Show when=move || prog_edit_id.get().is_some()>
                            <Button variant=ButtonVariant::Destructive on_click=delete_prog disabled=Signal::derive(move || saving.get())>
                                <IconTrash class="w-3.5 h-3.5" />
                                <span>"Hapus"</span>
                            </Button>
                        </Show>
                        <Button r#type="submit" class="ml-auto" disabled=Signal::derive(move || saving.get())>
                            {move || if saving.get() { "Menyimpan..." } else { "Simpan Program" }}
                        </Button>
                    </div>
                </form>
            </Dialog>

            // MODAL 4: Input Insiden / Lagging Bulanan
            <Dialog open=lag_m_open title="Input Kejadian / Statistik Bulanan">
                <form on:submit=save_lag_month class="space-y-4">
                    <div class="text-xs text-muted-foreground">
                        {move || lag_sel.get().map(|i| format!("{} ({})", i.parameter_name, month_short_name(lag_month.get()))).unwrap_or_default()}
                    </div>
                    <div class="space-y-1.5"><Label>"Jumlah Kejadian / Kasus"</Label><Input value=lag_val r#type="number" /></div>
                    <Show when=move || lag_sel.get().map(|i| i.indicator_key == "lti").unwrap_or(false)>
                        <div class="space-y-1.5"><Label>"Jumlah Hari Hilang (untuk SR)"</Label><Input value=lag_lost r#type="number" /></div>
                    </Show>
                    <div class="space-y-1.5"><Label>"Kronologi Singkat / Catatan"</Label><Textarea value=lag_notes /></div>
                    <div class="flex justify-end">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>"Simpan"</Button>
                    </div>
                </form>
            </Dialog>

            // MODAL 5: Analisis Akar Masalah Lagging
            <Dialog open=lag_meta_open title="Analisis Akar Masalah & Tindakan Korektif">
                <form on:submit=save_lag_meta class="space-y-4">
                    <div class="space-y-1.5"><Label>"Analisis Akar Masalah"</Label><Textarea value=lag_rca rows=3 /></div>
                    <div class="space-y-1.5"><Label>"Tindakan Korektif & Pencegahan"</Label><Textarea value=lag_capa rows=3 /></div>
                    <div class="flex justify-end">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>"Simpan Analisis"</Button>
                    </div>
                </form>
            </Dialog>
        </div>
    }
}
