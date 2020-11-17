use html::{classes, component, html};

#[derive(Clone)]
pub struct AlertProps {
	pub level: Level,
	pub title: Option<String>,
}

#[derive(Clone)]
pub enum Level {
	Info,
	Success,
	Warning,
	Danger,
}

#[component]
pub fn Alert(level: Level, title: Option<String>) {
	let level_class: String = match level {
		Level::Info => "alert-level-info".to_owned(),
		Level::Success => "alert-level-success".to_owned(),
		Level::Warning => "alert-level-warning".to_owned(),
		Level::Danger => "alert-level-danger".to_owned(),
	};
	let class = classes!("alert-wrapper", level_class);
	html! {
		<div class={class}>
		{
			 title.map(|title|  {
				html! {
					<div class="alert-title">
						{title}
					</div>
				}
			})
		}
		{children}
		</div>
	}
}
