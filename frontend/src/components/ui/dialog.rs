use crate::components::ui::icons::IconX;
use leptos::prelude::*;

#[component]
pub fn Dialog(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(optional, into)] max_width: String,
    children: ChildrenFn,
) -> impl IntoView {
    let mw = if max_width.is_empty() {
        "max-w-lg".to_string()
    } else {
        max_width
    };
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-end sm:items-center justify-center bg-black/50 backdrop-blur-xs p-0 sm:p-4">
                <div class=format!("w-full {} bg-card text-card-foreground rounded-t-2xl sm:rounded-xl border shadow-xl max-h-[90vh] flex flex-col overflow-hidden", mw)>
                    <div class="flex items-center justify-between px-5 py-4 border-b bg-muted/30">
                        <h3 class="font-semibold text-base text-foreground tracking-tight">{title.clone()}</h3>
                        <button
                            type="button"
                            class="rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer"
                            on:click=move |_| open.set(false)
                        >
                            <IconX class="w-4 h-4" />
                        </button>
                    </div>
                    <div class="p-5 sm:p-6 overflow-y-auto flex-1 space-y-4">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
