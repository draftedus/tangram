use html::{component, html, style};

#[component]
pub fn Token(color: Option<String>) {
	let style = style! {
		"background-color" => color,
	};
	html! {
		<div class="token" style={style}>
			{children}
		</div>
	}
}
