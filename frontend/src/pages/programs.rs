use crate::api::{
    api_create_program, api_delete_program, api_list_programs, api_update_program,
    api_update_program_month,
};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::Dialog;
use crate::components::ui::icons::{IconGrid, IconPencil, IconPlus, IconSearch, IconTable, IconTrash};
use crate::components::ui::input::{Input, Label, Textarea};
use crate::state::AppState;
use leptos::prelude::*;
use shared::formulas::month_short_name;
use shared::models::{
    UpdateProgramMonthRequest, UpsertWorkProgramRequest, WorkProgramDto, WorkProgramMonthDto,
};
use uuid::Uuid;

#[component]
pub fn ProgramsPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let programs = RwSignal::new(Vec::<WorkProgramDto>::new());
    let loading = RwSignal::new(true);
    let filter_pillar = RwSignal::new("ALL".to_string());
    let search_q = RwSignal::new(String::new());
    let matrix_mode = RwSignal::new(false);

    // Realization Modal
    let real_open = RwSignal::new(false);
    let real_prog = RwSignal::new(Option::<WorkProgramDto>::None);
    let real_month = RwSignal::new(1i32);
    let real_actual = RwSignal::new("0".to_string());
    let real_notes = RwSignal::new(String::new());

    // Master CRUD Modal
    let form_open = RwSignal::new(false);
    let editing_id = RwSignal::new(Option::<Uuid>::None);
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
            match api_list_programs(&period.id.to_string(), &token).await {
                Ok(list) => programs.set(list),
                Err(e) => state.notify(e, true),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        load();
    });

    let filtered = move || {
        let pil = filter_pillar.get();
        let q = search_q.get().to_lowercase();
        programs
            .get()
            .into_iter()
            .filter(|p| pil == "ALL" || p.pillar == pil)
            .filter(|p| {
                q.is_empty()
                    || p.code.to_lowercase().contains(&q)
                    || p.description.to_lowercase().contains(&q)
                    || p.pic.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };

    let can_master = move || {
        state
            .user
            .get()
            .map(|u| u.role.can_manage_master())
            .unwrap_or(false)
    };

    let open_new = Callback::new(move |_| {
        editing_id.set(None);
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
        form_open.set(true);
    });

    let open_edit = move |p: WorkProgramDto| {
        editing_id.set(Some(p.id));
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
        form_open.set(true);
    };

    let open_real = move |p: WorkProgramDto, m: i32| {
        let m_data = p.months.iter().find(|x| x.month == m).cloned();
        real_prog.set(Some(p));
        real_month.set(m);
        real_actual.set(format!("{}", m_data.as_ref().map(|x| x.actual_value).unwrap_or(0.0)));
        real_notes.set(m_data.map(|x| x.notes).unwrap_or_default());
        real_open.set(true);
    };

    let save_real = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(p) = real_prog.get_untracked() else {
            return;
        };
        let m = real_month.get_untracked();
        let req = UpdateProgramMonthRequest {
            plan_label: None,
            plan_value: None,
            actual_value: real_actual.get_untracked().parse().unwrap_or(0.0),
            notes: real_notes.get_untracked(),
        };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_program_month(&p.id.to_string(), m, &req, &token).await {
                Ok(updated) => {
                    programs.update(|list| {
                        if let Some(pos) = list.iter().position(|x| x.id == updated.id) {
                            list[pos] = updated;
                        }
                    });
                    real_open.set(false);
                    state.notify("Realisasi bulanan disimpan", false);
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let save_program = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(period) = state.active_period.get_untracked() else {
            return;
        };
        let pillar = f_pillar.get_untracked();
        let pillar_title = match pillar.as_str() {
            "A" => "A. KESELAMATAN KERJA (SAFETY) — MEMENUHI 5 UNSUR WAJIB PROGRAM P2K3 (PASAL 10 PERMENAKER 13/2025)".to_string(),
            "B" => "B. KESEHATAN KERJA, HIGIENE INDUSTRI & SANITAIR (HEALTH)".to_string(),
            "C" => "C. LINGKUNGAN HIDUP & PENGELOLAAN LIMBAH B3 (ENVIRONMENT)".to_string(),
            _ => "D. MUTU LAYANAN K3, SARANA PRASARANA & LINGKUNGAN KERJA INKLUSIF".to_string(),
        };
        let existing_months = editing_id
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
        let eid = editing_id.get_untracked();
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let res = match eid {
                Some(id) => api_update_program(&id.to_string(), &req, &token).await,
                None => api_create_program(&period.id.to_string(), &req, &token).await,
            };
            match res {
                Ok(saved) => {
                    programs.update(|list| {
                        if let Some(pos) = list.iter().position(|x| x.id == saved.id) {
                            list[pos] = saved;
                        } else {
                            list.push(saved);
                        }
                    });
                    form_open.set(false);
                    state.notify("Program kerja berhasil disimpan", false);
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let on_delete = move |id: Uuid| {
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete_program(&id.to_string(), &token).await {
                Ok(_) => {
                    programs.update(|list| list.retain(|p| p.id != id));
                    state.notify("Program kerja dihapus", false);
                }
                Err(e) => state.notify(e, true),
            }
        });
    };

    view! {
        <div class="space-y-4">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                <div>
                    <h1 class="text-xl md:text-2xl font-bold">"2. Matriks Program Kerja (TSP) K3L / P2K3"</h1>
                    <p class="text-xs md:text-sm text-muted-foreground">
                        "Leading Indicators Pilar A–D lengkap dengan jadwal Jan–Des, realisasi, dan analisis akar masalah."
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| matrix_mode.update(|v| *v = !*v))
                    >
                        {move || if matrix_mode.get() {
                            view! { <IconGrid class="w-3.5 h-3.5" /><span>"Mode Kartu"</span> }.into_any()
                        } else {
                            view! { <IconTable class="w-3.5 h-3.5" /><span>"Mode Tabel Matriks"</span> }.into_any()
                        }}
                    </Button>
                    <Show when=can_master>
                        <Button size=ButtonSize::Sm on_click=open_new>
                            <IconPlus class="w-3.5 h-3.5" />
                            <span>"Tambah Program"</span>
                        </Button>
                    </Show>
                </div>
            </div>

            // Filter Bar
            <div class="flex flex-col sm:flex-row gap-2">
                <div class="flex gap-1.5 overflow-x-auto pb-1">
                    {["ALL", "A", "B", "C", "D"].into_iter().map(|pil| {
                        let label = match pil {
                            "ALL" => "Semua Pilar",
                            "A" => "A. Safety",
                            "B" => "B. Health",
                            "C" => "C. Environment",
                            _ => "D. Mutu & Inklusif",
                        };
                        let p_val = pil.to_string();
                        let p_val2 = pil.to_string();
                        view! {
                            <button
                                type="button"
                                class=move || if filter_pillar.get() == p_val {
                                    "px-3 py-1.5 rounded-full text-xs font-semibold bg-primary text-primary-foreground whitespace-nowrap cursor-pointer"
                                } else {
                                    "px-3 py-1.5 rounded-full text-xs font-medium bg-card border hover:bg-muted whitespace-nowrap cursor-pointer"
                                }
                                on:click=move |_| filter_pillar.set(p_val2.clone())
                            >
                                {label}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <div class="sm:ml-auto sm:w-64 relative">
                    <IconSearch class="w-4 h-4 text-muted-foreground absolute left-3 top-3 pointer-events-none" />
                    <Input value=search_q placeholder="Cari kode, uraian, PIC..." class="pl-9" />
                </div>
            </div>

            <Show when=move || loading.get()>
                <div class="h-40 rounded-xl bg-muted animate-pulse"></div>
            </Show>

            // Mode Kartu (Mobile-First Default)
            <Show when=move || !matrix_mode.get()>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                    <For
                        each=filtered
                        key=|p| p.id
                        children=move |p| {
                            let p_edit = StoredValue::new(p.clone());
                            let p_real = p.clone();
                            let pid = p.id;
                            let badge_v = if p.achievement_pct >= 100.0 {
                                BadgeVariant::Success
                            } else if p.achievement_pct > 0.0 {
                                BadgeVariant::Warning
                            } else {
                                BadgeVariant::Secondary
                            };
                            view! {
                                <Card>
                                    <CardHeader class="pb-2">
                                        <div class="flex items-start justify-between gap-2">
                                            <div class="space-y-1">
                                                <div class="flex items-center gap-2 flex-wrap">
                                                    <Badge variant=BadgeVariant::Default>{p.code.clone()}</Badge>
                                                    <Badge variant=BadgeVariant::Outline>{format!("Pilar {}", p.pillar)}</Badge>
                                                    <Badge variant=badge_v>{format!("{:.0}% ({}/{})", p.achievement_pct, p.annual_realization, p.annual_target)}</Badge>
                                                </div>
                                                <CardTitle class="text-sm leading-snug pt-1">{p.description.clone()}</CardTitle>
                                            </div>
                                        </div>
                                    </CardHeader>
                                    <CardContent class="space-y-3 text-xs">
                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-1 text-muted-foreground">
                                            <div><strong>"PIC: "</strong>{p.pic.clone()}</div>
                                            <div><strong>"Frekuensi: "</strong>{p.frequency.clone()}</div>
                                            <div class="sm:col-span-2"><strong>"Output: "</strong>{p.output_evidence.clone()}</div>
                                        </div>
                                        // 12-Month Pill Grid
                                        <div class="grid grid-cols-6 sm:grid-cols-12 gap-1 pt-1">
                                            <For
                                                each=move || p.months.clone()
                                                key=|m| m.month
                                                children=move |m| {
                                                    let p_btn = p_real.clone();
                                                    let has_plan = !m.plan_label.is_empty();
                                                    let done = m.actual_value > 0.0;
                                                    let cls = if done {
                                                        "bg-emerald-100 text-emerald-800 border-emerald-300 font-semibold"
                                                    } else if m.plan_label == "Tentatif" {
                                                        "bg-amber-50 text-amber-800 border-amber-300"
                                                    } else if has_plan {
                                                        "bg-sky-50 text-sky-800 border-sky-300"
                                                    } else {
                                                        "bg-muted/30 text-muted-foreground border-transparent"
                                                    };
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=format!("rounded border p-1 text-[10px] text-center hover:ring-1 hover:ring-primary cursor-pointer {}", cls)
                                                            on:click=move |_| open_real(p_btn.clone(), m.month)
                                                        >
                                                            <div>{month_short_name(m.month)}</div>
                                                            <div class="font-bold">{format!("{:.0}", m.actual_value)}</div>
                                                        </button>
                                                    }
                                                }
                                            />
                                        </div>
                                        <div class="flex justify-end gap-2 pt-3 border-t">
                                            <Show when=can_master>
                                                <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(move |_| open_edit(p_edit.get_value()))>
                                                    <IconPencil class="w-3 h-3" />
                                                    <span>"Edit Master"</span>
                                                </Button>
                                                <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm on_click=Callback::new(move |_| on_delete(pid))>
                                                    <IconTrash class="w-3 h-3" />
                                                    <span>"Hapus"</span>
                                                </Button>
                                            </Show>
                                        </div>
                                    </CardContent>
                                </Card>
                            }
                        }
                    />
                </div>
            </Show>

            // Mode Tabel Matriks Lengkap (Desktop / Format HTML)
            <Show when=move || matrix_mode.get()>
                <Card class="overflow-hidden">
                    <div class="overflow-x-auto">
                        <table class="w-full text-[11px] border-collapse">
                            <thead>
                                <tr class="bg-primary text-primary-foreground">
                                    <th class="p-2.5 border border-emerald-800">"Kode"</th>
                                    <th class="p-2.5 border border-emerald-800 min-w-[220px] text-left">"Uraian Program Kerja"</th>
                                    <th class="p-2.5 border border-emerald-800 min-w-[140px] text-left">"PIC"</th>
                                    {(1..=12).map(|m| view! {
                                        <th class="p-2 border border-emerald-800 text-center">{month_short_name(m)}</th>
                                    }).collect_view()}
                                    <th class="p-2.5 border border-emerald-800">"Target"</th>
                                    <th class="p-2.5 border border-emerald-800">"Real"</th>
                                    <th class="p-2.5 border border-emerald-800">"Capaian"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=filtered
                                    key=|p| p.id
                                    children=move |p| {
                                        let p_real = p.clone();
                                        view! {
                                            <tr class="border-b hover:bg-muted/30">
                                                <td class="p-2 border font-semibold text-center text-primary">{p.code.clone()}</td>
                                                <td class="p-2 border">{p.description.clone()}</td>
                                                <td class="p-2 border text-muted-foreground">{p.pic.clone()}</td>
                                                <For
                                                    each=move || p.months.clone()
                                                    key=|m| m.month
                                                    children=move |m| {
                                                        let p_btn = p_real.clone();
                                                        let bg = if m.actual_value > 0.0 {
                                                            "bg-emerald-50 text-emerald-800 font-semibold"
                                                        } else if m.plan_label == "Tentatif" {
                                                            "bg-amber-50 text-amber-800"
                                                        } else if !m.plan_label.is_empty() {
                                                            "bg-sky-50 text-sky-800"
                                                        } else {
                                                            ""
                                                        };
                                                        view! {
                                                            <td
                                                                class=format!("p-1 border text-center cursor-pointer hover:bg-accent {}", bg)
                                                                on:click=move |_| open_real(p_btn.clone(), m.month)
                                                            >
                                                                <div>{if m.plan_label.is_empty() { "-".to_string() } else { m.plan_label.clone() }}</div>
                                                                <Show when=move || { m.actual_value > 0.0 }>
                                                                    <div class="text-[10px] font-bold">{format!("({:.0})", m.actual_value)}</div>
                                                                </Show>
                                                            </td>
                                                        }
                                                    }
                                                />
                                                <td class="p-2 border text-right tabular-nums">{format!("{:.0}", p.annual_target)}</td>
                                                <td class="p-2 border text-right tabular-nums font-semibold">{format!("{:.0}", p.annual_realization)}</td>
                                                <td class="p-2 border text-right tabular-nums font-bold">{format!("{:.0}%", p.achievement_pct)}</td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </div>
                </Card>
            </Show>

            // Dialog Input Realisasi Bulanan
            <Dialog open=real_open title="Input Realisasi Bulanan">
                <form on:submit=save_real class="space-y-4">
                    <div class="text-xs text-muted-foreground">
                        {move || real_prog.get().map(|p| format!("{} — {} ({})", p.code, p.description, month_short_name(real_month.get()))).unwrap_or_default()}
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Nilai Realisasi Bulan Ini"</Label>
                        <Input value=real_actual r#type="number" />
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Catatan / Link Bukti Dukung"</Label>
                        <Textarea value=real_notes placeholder="Contoh: Sudah dilaksanakan, logbook terlampir" />
                    </div>
                    <div class="flex justify-end gap-2">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                            "Simpan Realisasi"
                        </Button>
                    </div>
                </form>
            </Dialog>

            // Dialog Master CRUD Program Kerja
            <Dialog open=form_open title="Form Program Kerja K3L / P2K3" max_width="max-w-2xl">
                <form on:submit=save_program class="space-y-3">
                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                        <div class="space-y-1">
                            <Label>"Pilar (A/B/C/D)"</Label>
                            <Input value=f_pillar placeholder="A" />
                        </div>
                        <div class="space-y-1">
                            <Label>"Kode Program"</Label>
                            <Input value=f_code placeholder="A.2.16" />
                        </div>
                        <div class="space-y-1">
                            <Label>"Target Tahunan"</Label>
                            <Input value=f_target r#type="number" />
                        </div>
                    </div>
                    <div class="space-y-1">
                        <Label>"Sub-kategori (opsional)"</Label>
                        <Input value=f_subcat />
                    </div>
                    <div class="space-y-1">
                        <Label>"Tujuan"</Label>
                        <Input value=f_objective />
                    </div>
                    <div class="space-y-1">
                        <Label>"Sasaran Terukur"</Label>
                        <Textarea value=f_measurable rows=2 />
                    </div>
                    <div class="space-y-1">
                        <Label>"Uraian Program Kerja"</Label>
                        <Textarea value=f_desc rows=3 />
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1">
                            <Label>"Frekuensi Pelaksanaan"</Label>
                            <Input value=f_freq />
                        </div>
                        <div class="space-y-1">
                            <Label>"Penanggung Jawab (PIC)"</Label>
                            <Input value=f_pic />
                        </div>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1">
                            <Label>"Referensi / Dasar Hukum"</Label>
                            <Input value=f_ref />
                        </div>
                        <div class="space-y-1">
                            <Label>"Output / Bukti Dukung"</Label>
                            <Input value=f_output />
                        </div>
                    </div>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="space-y-1">
                            <Label>"Analisis Akar Masalah (<100%)"</Label>
                            <Textarea value=f_rca rows=2 />
                        </div>
                        <div class="space-y-1">
                            <Label>"Tindakan Perbaikan & Pencegahan"</Label>
                            <Textarea value=f_capa rows=2 />
                        </div>
                    </div>
                    <div class="flex justify-end gap-2 pt-2">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                            {move || if saving.get() { "Menyimpan..." } else { "Simpan Program" }}
                        </Button>
                    </div>
                </form>
            </Dialog>
        </div>
    }
}
