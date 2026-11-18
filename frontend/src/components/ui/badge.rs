use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Success,
    Warning,
    Destructive,
    Outline,
}

#[component]
pub fn Badge(
    #[prop(optional)] variant: BadgeVariant,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors";
    let v_cls = match variant {
        BadgeVariant::Default => "border-transparent bg-primary text-primary-foreground",
        BadgeVariant::Secondary => "border-transparent bg-secondary text-secondary-foreground",
        BadgeVariant::Success => "border-transparent bg-emerald-100 text-emerald-800",
        BadgeVariant::Warning => "border-transparent bg-amber-100 text-amber-800",
        BadgeVariant::Destructive => "border-transparent bg-red-100 text-red-800",
        BadgeVariant::Outline => "text-foreground border-border",
    };
    view! {
        <span class=format!("{} {} {}", base, v_cls, class)>
            {children()}
        </span>
    }
}
