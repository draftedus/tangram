use html::{component, html};

#[component]
pub fn FieldLabel(html_for: Option<String>) {
	html! {
		<label class="field-label" html_for={html_for}>
			{children}
		</label>
	}
}
