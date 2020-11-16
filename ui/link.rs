use html::{component, html};

#[component]
pub fn Link(class_name: Option<String>, href: Option<String>, title: Option<String>) {
	let class_name = if let Some(class_name) = class_name {
		format!("link {}", class_name)
	} else {
		"link".into()
	};
	html! {
		<a class={class_name} href={href} title={title}>
			{children}
		</a>
	}
}
