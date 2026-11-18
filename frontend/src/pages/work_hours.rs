use crate::api::{api_get_work_hours, api_update_work_hour};
use crate::components::ui::button::{Button, ButtonSize};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::components::ui::dialog::Dialog;
use crate::components::ui::icons::IconPencil;
use crate::components::ui::input::{Input, Label};
use crate::state::AppState;
use leptos::prelude::*;
use shared::formulas::month_short_name;
use shared::models::{UpdateWorkHoursMonthRequest, WorkHoursMonthDto, WorkHoursSummaryDto};

#[component]
pub fn WorkHoursPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let data = RwSignal::new(Option::<WorkHoursSummaryDto>::None);
    let loading = RwSignal::new(true);

    // Edit Modal State
    let edit_open = RwSignal::new(false);
    let edit_month = RwSignal::new(1i32);
    let edit_days = RwSignal::new(String::new());
    let edit_hours = RwSignal::new(String::new());
    let edit_employees = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let load = move || {
        let Some(token) = state.token.get() else {
            return;
        };
        let Some(period) = state.active_period.get() else {
            return;
        };
        let pid = period.id.to_string();
        loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_get_work_hours(&pid, &token).await {
                Ok(res) => data.set(Some(res)),
                Err(e) => state.notify(e, true),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        load();
    });

    let open_edit = move |m: WorkHoursMonthDto| {
        edit_month.set(m.month);
        edit_days.set(format!("{}", m.effective_days));
        edit_hours.set(format!("{}", m.standard_hours_per_day));
        edit_employees.set(format!("{}", m.avg_employees_present));
        edit_open.set(true);
    };

    let save_edit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(token) = state.token.get_untracked() else {
            return;
        };
        let Some(period) = state.active_period.get_untracked() else {
            return;
        };
        let m = edit_month.get_untracked();
        let req = UpdateWorkHoursMonthRequest {
            effective_days: edit_days.get_untracked().parse().unwrap_or(0.0),
            standard_hours_per_day: edit_hours.get_untracked().parse().unwrap_or(8.0),
            avg_employees_present: edit_employees.get_untracked().parse().unwrap_or(0.0),
        };
        saving.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api_update_work_hour(&period.id.to_string(), m, &req, &token).await {
                Ok(updated) => {
                    data.set(Some(updated));
                    edit_open.set(false);
                    state.notify("Data jam kerja berhasil diperbarui", false);
                }
                Err(e) => state.notify(e, true),
            }
            saving.set(false);
        });
    };

    let can_edit = move || {
        state
            .user
            .get()
            .map(|u| u.role.can_manage_master())
            .unwrap_or(false)
    };

    view! {
        <div class="space-y-6">
            <div class="space-y-1">
                <h1 class="text-xl md:text-2xl font-bold tracking-tight">"1. Jam Kerja Efektif & Safe Manhours 2027"</h1>
                <p class="text-xs md:text-sm text-muted-foreground">
                    "Hari kerja efektif ASN (SKB 3 Menteri: 242 hari). Isi rata-rata pegawai hadir per hari untuk menghitung Jam Kerja Aman otomatis."
                </p>
            </div>

            {move || data.get().map(|wh| {
                let months_sv = StoredValue::new(wh.months.clone());
                view! {
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <Card>
                        <CardContent class="space-y-1">
                            <div class="text-xs text-muted-foreground">"Total Hari Kerja"</div>
                            <div class="text-xl font-bold text-primary tabular-nums">{format!("{:.0} hari", wh.total_effective_days)}</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent class="space-y-1">
                            <div class="text-xs text-muted-foreground">"Rata-rata Jam/Hari"</div>
                            <div class="text-xl font-bold tabular-nums">{format!("{:.1} jam", wh.avg_standard_hours)}</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent class="space-y-1">
                            <div class="text-xs text-muted-foreground">"Rata-rata Pegawai/Hari"</div>
                            <div class="text-xl font-bold tabular-nums">{format!("{:.1} org", wh.avg_employees_present)}</div>
                        </CardContent>
                    </Card>
                    <Card class="bg-primary text-primary-foreground border-primary">
                        <CardContent class="space-y-1">
                            <div class="text-xs text-emerald-100">"Safe Manhours Kumulatif"</div>
                            <div class="text-xl font-bold tabular-nums">{format!("{:.0} jam", wh.total_cumulative_hours)}</div>
                        </CardContent>
                    </Card>
                </div>

                // Mobile Card View (< md)
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 md:hidden">
                    <For
                        each=move || months_sv.get_value()
                        key=|m| m.month
                        children=move |m| {
                            let m_clone = StoredValue::new(m.clone());
                            view! {
                                <Card>
                                    <CardHeader class="flex flex-row items-center justify-between border-b bg-muted/20">
                                        <CardTitle class="text-sm">{format!("{} 2027", month_short_name(m.month))}</CardTitle>
                                        <Show when=can_edit>
                                            <Button size=ButtonSize::Sm on_click=Callback::new(move |_| open_edit(m_clone.get_value()))>
                                                <IconPencil class="w-3 h-3" />
                                                <span>"Edit"</span>
                                            </Button>
                                        </Show>
                                    </CardHeader>
                                    <CardContent class="text-xs space-y-2">
                                        <div class="flex justify-between"><span class="text-muted-foreground">"Hari Kerja Efektif:"</span><span class="font-medium tabular-nums">{format!("{:.0} hari", m.effective_days)}</span></div>
                                        <div class="flex justify-between"><span class="text-muted-foreground">"Standar Jam/Hari:"</span><span class="font-medium tabular-nums">{format!("{:.1} jam", m.standard_hours_per_day)}</span></div>
                                        <div class="flex justify-between"><span class="text-muted-foreground">"Rata-rata Hadir:"</span><span class="font-semibold text-primary tabular-nums">{format!("{:.1} org", m.avg_employees_present)}</span></div>
                                        <div class="flex justify-between border-t pt-2"><span class="font-semibold">"Kumulatif Bulan:"</span><span class="font-bold text-primary tabular-nums">{format!("{:.0} jam", m.cumulative_monthly_hours)}</span></div>
                                    </CardContent>
                                </Card>
                            }
                        }
                    />
                </div>

                // Desktop Table View (>= md)
                <Card class="hidden md:block overflow-hidden">
                    <div class="overflow-x-auto">
                        <table class="w-full text-xs border-collapse">
                            <thead>
                                <tr class="bg-primary text-primary-foreground">
                                    <th class="p-3 text-left border border-emerald-800">"Parameter Perhitungan"</th>
                                    <For
                                        each=move || months_sv.get_value()
                                        key=|m| m.month
                                        children=move |m| {
                                            let m_clone = StoredValue::new(m.clone());
                                            view! {
                                                <th class="p-2.5 text-center border border-emerald-800">
                                                    <div>{month_short_name(m.month)}</div>
                                                    <Show when=can_edit>
                                                        <button
                                                            type="button"
                                                            class="mt-1 inline-flex items-center gap-1 text-[10px] underline text-emerald-100 hover:text-white cursor-pointer"
                                                            on:click=move |_| open_edit(m_clone.get_value())
                                                        >
                                                            <IconPencil class="w-2.5 h-2.5" />
                                                            <span>"Edit"</span>
                                                        </button>
                                                    </Show>
                                                </th>
                                            }
                                        }
                                    />
                                    <th class="p-3 text-center border border-emerald-800">"Total / Kumulatif"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr class="border-b">
                                    <td class="p-3 font-medium">"Hari kerja efektif ASN per bulan (hari)"</td>
                                    <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.0}", m.effective_days)}</td> } />
                                    <td class="p-2.5 text-right font-bold tabular-nums border-l">{format!("{:.0}", wh.total_effective_days)}</td>
                                </tr>
                                <tr class="border-b">
                                    <td class="p-3 font-medium">"Standar jam kerja efektif per hari (jam)"</td>
                                    <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.0}", m.standard_hours_per_day)}</td> } />
                                    <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.1}", wh.avg_standard_hours)}</td>
                                </tr>
                                <tr class="border-b bg-emerald-50/50">
                                    <td class="p-3 font-semibold text-primary">"Rata-rata jumlah pegawai hadir per hari (orang)"</td>
                                    <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2.5 text-right font-semibold text-primary tabular-nums border-l">{format!("{:.0}", m.avg_employees_present)}</td> } />
                                    <td class="p-2.5 text-right font-semibold tabular-nums border-l">{format!("{:.1}", wh.avg_employees_present)}</td>
                                </tr>
                                <tr class="border-b bg-muted/40 font-semibold">
                                    <td class="p-3">"Total jam kerja seluruh pegawai per hari (jam)"</td>
                                    <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.0}", m.total_daily_hours)}</td> } />
                                    <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.0}", wh.sum_daily_hours)}</td>
                                </tr>
                                <tr class="bg-accent/70 font-bold text-primary">
                                    <td class="p-3">"Total jam kerja kumulatif per bulan (jam)"</td>
                                    <For each=move || months_sv.get_value() key=|m| m.month children=|m| view! { <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.0}", m.cumulative_monthly_hours)}</td> } />
                                    <td class="p-2.5 text-right tabular-nums border-l">{format!("{:.0}", wh.total_cumulative_hours)}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </Card>
                }
            })}

            <Dialog open=edit_open title="Edit Jam Kerja Bulanan">
                <form on:submit=save_edit class="space-y-4">
                    <div class="text-sm font-semibold text-primary">
                        {move || format!("Bulan: {} 2027", month_short_name(edit_month.get()))}
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Hari Kerja Efektif ASN (hari)"</Label>
                        <Input value=edit_days r#type="number" />
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Standar Jam Kerja per Hari (jam)"</Label>
                        <Input value=edit_hours r#type="number" />
                    </div>
                    <div class="space-y-1.5">
                        <Label>"Rata-rata Jumlah Pegawai Hadir per Hari (orang)"</Label>
                        <Input value=edit_employees r#type="number" />
                    </div>
                    <div class="flex justify-end gap-2 pt-2">
                        <Button r#type="submit" disabled=Signal::derive(move || saving.get())>
                            {move || if saving.get() { "Menyimpan..." } else { "Simpan" }}
                        </Button>
                    </div>
                </form>
            </Dialog>
        </div>
    }
}
