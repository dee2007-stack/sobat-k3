use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
    Icon,
}

#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] r#type: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 cursor-pointer";
    let v_cls = match variant {
        ButtonVariant::Primary => "bg-primary text-primary-foreground hover:bg-primary/90 shadow-xs",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Outline => "border border-input bg-card hover:bg-accent hover:text-accent-foreground shadow-2xs",
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Destructive => "bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-xs",
    };
    let s_cls = match size {
        ButtonSize::Sm => "h-8 px-3 text-xs",
        ButtonSize::Md => "h-10 px-4 py-2",
        ButtonSize::Lg => "h-11 px-6 text-base",
        ButtonSize::Icon => "h-9 w-9",
    };
    let btn_type = if r#type.is_empty() {
        "button".to_string()
    } else {
        r#type
    };
    let full_cls = format!("{} {} {} {}", base, v_cls, s_cls, class);

    view! {
        <button
            type=btn_type
            class=full_cls
            disabled=move || disabled.get()
            on:click=move |ev| {
                if let Some(cb) = &on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}
