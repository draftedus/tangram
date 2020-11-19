use html::{component, html};

#[derive(Clone)]
pub struct DetailsOption {
	pub href: String,
	pub title: String,
}

#[component]
pub fn Details(options: Option<Vec<DetailsOption>>, summary: Option<String>) {
	html! {
		<details class="details">
			<summary class="details-summary" role="button">
				{summary}
			</summary>
			<div class="details-list">
				{options.map(|options|
					options.into_iter().map(|option| {
						html! {
							<a class="details-list-item" href={option.href} key={option.title.clone()}>
							{option.title}
							</a>
						}
					}).collect::<Vec<_>>()
				)}
			</div>
		</details>
	}
}
