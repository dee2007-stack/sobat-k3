use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::icons::{
    IconClipboardList, IconClock, IconFileCheck, IconLayoutDashboard, IconLogOut, IconMenu,
    IconSettings, IconShieldCheck,
};
use crate::state::AppState;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use shared::models::UserRole;

#[component]
pub fn ToastContainer() -> impl IntoView {
    let state = expect_context::<AppState>();
    view! {
        <div class="fixed top-4 right-4 z-[60] flex flex-col gap-2 max-w-xs w-full pointer-events-none no-print">
            <For
                each=move || state.toasts.get()
                key=|t| t.id
                children=move |t| {
                    let cls = if t.is_error {
                        "bg-destructive text-destructive-foreground border-destructive"
                    } else {
                        "bg-card text-foreground border-emerald-200"
                    };
                    view! {
                        <div class=format!("pointer-events-auto rounded-lg border px-4 py-3 text-sm font-medium shadow-lg {}", cls)>
                            {t.title}
                        </div>
                    }
                }
            />
        </div>
    }
}

fn is_route_active(pathname: &str, target: &str) -> bool {
    let norm = pathname.trim_end_matches('/');
    let norm_target = target.trim_end_matches('/');
    norm == norm_target
}

fn render_nav_icon(path: &str) -> AnyView {
    match path {
        "/sobat-k3/" => view! { <IconLayoutDashboard class="w-4 h-4 shrink-0" /> }.into_any(),
        "/sobat-k3/work-hours" => view! { <IconClock class="w-4 h-4 shrink-0" /> }.into_any(),
        "/sobat-k3/programs" => view! { <IconClipboardList class="w-4 h-4 shrink-0" /> }.into_any(),
        "/sobat-k3/lagging" => view! { <IconShieldCheck class="w-4 h-4 shrink-0" /> }.into_any(),
        "/sobat-k3/report" => view! { <IconFileCheck class="w-4 h-4 shrink-0" /> }.into_any(),
        _ => view! { <IconSettings class="w-4 h-4 shrink-0" /> }.into_any(),
    }
}

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let state = expect_context::<AppState>();
    let location = use_location();
    let mobile_menu_open = RwSignal::new(false);

    let nav_items = move || {
        let is_admin = state
            .user
            .get()
            .map(|u| u.role == UserRole::Admin)
            .unwrap_or(false);
        let mut items = vec![
            ("/sobat-k3/", "Dashboard"),
            ("/sobat-k3/work-hours", "Jam Kerja"),
            ("/sobat-k3/programs", "Program TSP"),
            ("/sobat-k3/lagging", "Lagging K3L"),
            ("/sobat-k3/report", "Pengesahan"),
        ];
        if is_admin {
            items.push(("/sobat-k3/admin", "Manajemen Admin"));
        }
        items
    };

    let on_logout = Callback::new(move |_| {
        state.logout();
    });

    view! {
        <div class="min-h-screen flex flex-col md:flex-row bg-background">
            // Desktop Sidebar
            <aside class="hidden md:flex md:w-64 md:flex-col md:fixed md:inset-y-0 border-r bg-card no-print">
                <div class="flex items-center gap-3 px-5 py-4 border-b bg-primary text-primary-foreground">
                    <div class="h-9 w-9 rounded-lg bg-white/15 flex items-center justify-center shrink-0">
                        <IconShieldCheck class="w-5 h-5 text-white" />
                    </div>
                    <div class="min-w-0">
                        <div class="font-bold text-sm leading-tight tracking-tight">"SOBAT K3"</div>
                        <div class="text-[11px] text-emerald-100 truncate">"P2K3 BBPOM Bandung"</div>
                    </div>
                </div>
                <nav class="flex-1 p-3 space-y-1 overflow-y-auto">
                    <For
                        each=nav_items
                        key=|item| item.0
                        children=move |(path, label)| {
                            let is_active = move || is_route_active(&location.pathname.get(), path);
                            view! {
                                <A href=path attr:class=move || {
                                    if is_active() {
                                        "flex items-center gap-3 rounded-lg px-3.5 py-2.5 text-sm font-semibold bg-accent text-accent-foreground transition-colors"
                                    } else {
                                        "flex items-center gap-3 rounded-lg px-3.5 py-2.5 text-sm font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
                                    }
                                }>
                                    {render_nav_icon(path)}
                                    <span>{label}</span>
                                </A>
                            }
                        }
                    />
                </nav>
                <div class="p-4 border-t bg-muted/20 space-y-3">
                    {move || state.user.get().map(|u| view! {
                        <div class="space-y-1.5">
                            <div class="text-sm font-semibold truncate">{u.full_name}</div>
                            <div class="text-xs text-muted-foreground truncate">{u.position_title}</div>
                            <Badge variant=BadgeVariant::Secondary>{u.role.display_name()}</Badge>
                        </div>
                    })}
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm class="w-full" on_click=on_logout>
                        <IconLogOut class="w-3.5 h-3.5" />
                        <span>"Keluar"</span>
                    </Button>
                </div>
            </aside>

            // Mobile Top Header
            <header class="md:hidden sticky top-0 z-30 flex items-center justify-between px-4 py-3 bg-primary text-primary-foreground shadow-sm no-print">
                <div class="flex items-center gap-2.5">
                    <div class="h-8 w-8 rounded-lg bg-white/15 flex items-center justify-center">
                        <IconShieldCheck class="w-4 h-4 text-white" />
                    </div>
                    <div>
                        <div class="font-bold text-sm leading-tight">"Sobat K3"</div>
                        <div class="text-[10px] text-emerald-100">"BBPOM Bandung • 2027"</div>
                    </div>
                </div>
                <button
                    type="button"
                    class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium bg-white/15 hover:bg-white/25 transition-colors cursor-pointer"
                    on:click=move |_| mobile_menu_open.update(|v| *v = !*v)
                >
                    <IconMenu class="w-4 h-4" />
                    <span>{move || state.user.get().map(|u| u.role.display_name()).unwrap_or("Menu")}</span>
                </button>
            </header>

            // Mobile Profile Drawer
            <Show when=move || mobile_menu_open.get()>
                <div class="md:hidden fixed inset-0 z-40 bg-black/40 backdrop-blur-xs no-print" on:click=move |_| mobile_menu_open.set(false)>
                    <div class="absolute right-3 top-14 w-68 rounded-xl border bg-card p-4 shadow-xl space-y-3" on:click=|ev| ev.stop_propagation()>
                        {move || state.user.get().map(|u| view! {
                            <div class="space-y-1.5 border-b pb-3">
                                <div class="font-semibold text-sm">{u.full_name}</div>
                                <div class="text-xs text-muted-foreground">{u.email}</div>
                                <Badge variant=BadgeVariant::Default>{u.role.display_name()}</Badge>
                            </div>
                        })}
                        <div class="flex flex-col gap-1">
                            <A href="/sobat-k3/report" attr:class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm hover:bg-muted" on:click=move |_| mobile_menu_open.set(false)>
                                <IconFileCheck class="w-4 h-4 text-primary" />
                                <span>"Pengesahan & Export"</span>
                            </A>
                            <Show when=move || state.user.get().map(|u| u.role == UserRole::Admin).unwrap_or(false)>
                                <A href="/sobat-k3/admin" attr:class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm hover:bg-muted" on:click=move |_| mobile_menu_open.set(false)>
                                    <IconSettings class="w-4 h-4 text-primary" />
                                    <span>"Manajemen Pengguna"</span>
                                </A>
                            </Show>
                        </div>
                        <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm class="w-full" on_click=on_logout>
                            <IconLogOut class="w-3.5 h-3.5" />
                            <span>"Keluar"</span>
                        </Button>
                    </div>
                </div>
            </Show>

            // Main Content Area
            <main class="flex-1 min-w-0 w-full md:pl-64 pb-24 md:pb-10 overflow-x-hidden print-full">
                <div class="max-w-7xl mx-auto px-4 py-5 sm:px-6 sm:py-6 lg:px-8 space-y-6 print-full">
                    {children()}
                </div>
            </main>

            // Mobile Bottom Navigation Bar
            <nav class="md:hidden fixed bottom-0 inset-x-0 z-30 border-t bg-card/95 backdrop-blur-md flex justify-around items-center h-16 px-2 no-print">
                <A href="/sobat-k3/" attr:class=move || if is_route_active(&location.pathname.get(), "/sobat-k3/") { "flex flex-col items-center gap-1 text-[11px] font-semibold text-primary" } else { "flex flex-col items-center gap-1 text-[11px] text-muted-foreground" }>
                    <IconLayoutDashboard class="w-5 h-5" />
                    <span>"Home"</span>
                </A>
                <A href="/sobat-k3/work-hours" attr:class=move || if is_route_active(&location.pathname.get(), "/sobat-k3/work-hours") { "flex flex-col items-center gap-1 text-[11px] font-semibold text-primary" } else { "flex flex-col items-center gap-1 text-[11px] text-muted-foreground" }>
                    <IconClock class="w-5 h-5" />
                    <span>"Jam Kerja"</span>
                </A>
                <A href="/sobat-k3/programs" attr:class=move || if is_route_active(&location.pathname.get(), "/sobat-k3/programs") { "flex flex-col items-center gap-1 text-[11px] font-semibold text-primary" } else { "flex flex-col items-center gap-1 text-[11px] text-muted-foreground" }>
                    <IconClipboardList class="w-5 h-5" />
                    <span>"TSP"</span>
                </A>
                <A href="/sobat-k3/lagging" attr:class=move || if is_route_active(&location.pathname.get(), "/sobat-k3/lagging") { "flex flex-col items-center gap-1 text-[11px] font-semibold text-primary" } else { "flex flex-col items-center gap-1 text-[11px] text-muted-foreground" }>
                    <IconShieldCheck class="w-5 h-5" />
                    <span>"Lagging"</span>
                </A>
                <A href="/sobat-k3/report" attr:class=move || if is_route_active(&location.pathname.get(), "/sobat-k3/report") { "flex flex-col items-center gap-1 text-[11px] font-semibold text-primary" } else { "flex flex-col items-center gap-1 text-[11px] text-muted-foreground" }>
                    <IconFileCheck class="w-5 h-5" />
                    <span>"Laporan"</span>
                </A>
            </nav>
        </div>
    }
}
