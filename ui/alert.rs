use html::{component, html};

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
		Level::Info => "alert-level-info".into(),
		Level::Success => "alert-level-success".into(),
		Level::Warning => "alert-level-warning".into(),
		Level::Danger => "alert-level-danger".into(),
	};
	html! {
		<div class={format!("alert-wrapper {}", level_class)}>
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
