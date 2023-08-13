use leptos::*;

#[component]
pub fn ErrorTemplate(cx: Scope, error: ServerFnError) -> impl IntoView {
    view! {cx,
      <pre class="error">{format!("{}", error)}</pre>
    }
}
