use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("rounded-xl border border-border/90 bg-card text-card-foreground shadow-sm {}", class)>
            {children()}
        </div>
    }
}

#[component]
pub fn CardHeader(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("flex flex-col space-y-1.5 px-5 pt-5 pb-3 sm:px-6 sm:pt-6 sm:pb-3 {}", class)>
            {children()}
        </div>
    }
}

#[component]
pub fn CardTitle(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <h3 class=format!("text-base font-semibold leading-snug tracking-tight {}", class)>
            {children()}
        </h3>
    }
}

#[component]
pub fn CardContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("p-5 sm:p-6 {}", class)>
            {children()}
        </div>
    }
}
