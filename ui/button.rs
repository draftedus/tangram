use html::{component, html};

#[derive(Clone)]
pub enum ButtonType {
	Submit,
	Button,
	Reset,
}

#[component]
pub fn Button(
	disabled: Option<bool>,
	download: Option<String>,
	href: Option<String>,
	id: Option<String>,
	button_type: ButtonType,
) {
	let button_type = match button_type {
		ButtonType::Submit => "submit".to_owned(),
		ButtonType::Button => "button".to_owned(),
		ButtonType::Reset => "reset".to_owned(),
	};
	if let Some(href) = href {
		html! {
			<a
				class="button"
				disabled={disabled}
				download={download}
				href={href}
				type={button_type}
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
				type={button_type}
			>
				{children}
			</button>
		}
	}
}
