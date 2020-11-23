use html::{component, html, style};

pub enum WindowShade {
	Code,
	Default,
}

#[component]
pub fn Window() {
	let red_style = style! {
		"background-color" => "var(--red)",
	};
	let yellow_style = style! {
		"background-color"=> "var(--yellow)",
	};
	let green_style = style! {
		"background-color"=> "var(--green)",
	};
	html! {
		<div class="window-wrapper">
			<div class="window-topbar">
				<div
					class="window-topbar-button"
					style={red_style}
				/>
				<div
					class="window-topbar-button"
					style={yellow_style}
				/>
				<div
					class="window-topbar-button"
					style={green_style}
				/>
			</div>
			<div class="window-body">{children}</div>
		</div>
	}
}
