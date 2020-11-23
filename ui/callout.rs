use crate::alert::Level;
use html::{classes, component, html};

#[component]
pub fn Callout(level: Level, title: Option<String>) {
	let level_class = match level {
		Level::Danger => "callout-wrapper-danger",
		Level::Info => "callout-wrapper-info",
		Level::Warning => "callout-wrapper-warning",
		Level::Success => "callout-wrapper-success",
	};
	let class = classes!("callout-wrapper", level_class);
	html! {
			<div class={class}>
				{
					title.map(|title|
						html! {
							<div class="callout-title">{title}</div>
						}
					)
				}
				<div class="callout-inner">{children}</div>
			</div>
	}
}
