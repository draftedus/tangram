use html::{component, html, style};

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
	color: Option<String>,
) {
	let button_type = match button_type {
		ButtonType::Submit => "submit",
		ButtonType::Button => "button",
		ButtonType::Reset => "reset",
	};
	let style = style! {
		"background-color" => color,
	};
	if let Some(href) = href {
		html! {
			<a
				class="button"
				disabled={disabled}
				download={download}
				href={href}
				style={style}
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
				style={style}
				type={button_type}
			>
				{children}
			</button>
		}
	}
}
