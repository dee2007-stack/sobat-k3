use leptos::prelude::*;

#[component]
pub fn Label(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <label class=format!("text-xs md:text-sm font-medium leading-none text-foreground {}", class)>
            {children()}
        </label>
    }
}

#[component]
pub fn Input(
    #[prop(into)] value: RwSignal<String>,
    #[prop(optional, into)] r#type: String,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let input_type = if r#type.is_empty() {
        "text".to_string()
    } else {
        r#type
    };
    view! {
        <input
            type=input_type
            placeholder=placeholder
            disabled=disabled
            class=format!("flex h-10 w-full rounded-md border border-input bg-card px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 {}", class)
            prop:value=move || value.get()
            on:input=move |ev| value.set(event_target_value(&ev))
        />
    }
}

#[component]
pub fn Textarea(
    #[prop(into)] value: RwSignal<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] class: String,
    #[prop(optional)] rows: u32,
) -> impl IntoView {
    let r = if rows == 0 { 3 } else { rows };
    view! {
        <textarea
            rows=r
            placeholder=placeholder
            class=format!("flex w-full rounded-md border border-input bg-card px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring {}", class)
            prop:value=move || value.get()
            on:input=move |ev| value.set(event_target_value(&ev))
        />
    }
}
