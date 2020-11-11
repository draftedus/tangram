use html::{component, html};

#[component]
pub fn Button(
	disabled: Option<bool>,
	download: Option<String>,
	href: Option<String>,
	id: Option<String>,
) {
	if let Some(href) = href {
		html! {
			<a
				class="button"
				disabled={disabled}
				download={download}
				href={href}
			>
				{children}
			</a>
		}
	} else {
		html! {
			<button
				class="button"
				disabled={disabled}
				id={id}
			>
				{children}
			</button>
		}
	}
}
