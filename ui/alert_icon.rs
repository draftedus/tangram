use super::alert::Level;
use html::{classes, component, html};

#[component]
pub fn AlertIcon(alert: String, level: Level) {
	let level_class = match level {
		Level::Info => "alert-icon-level-info",
		Level::Success => "alert-icon-level-success",
		Level::Warning => "alert-icon-level-warning",
		Level::Danger => "alert-icon-level-danger",
	};
	let alertMessageClass = classes!("alert-icon-message", level_class);
	let alertIconClass = classes!("alert-icon", level_class);
	html! {
		<div class="alert-icon-container">
			<div class={alertMessageClass}>{alert}</div>
			<div class={alertIconClass}>{children}</div>
		</div>
	}
}
