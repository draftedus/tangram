use html::{classes, component, html};

#[component]
pub fn Link(class: Option<String>, href: Option<String>, title: Option<String>) {
	let class = classes!("link", class);
	html! {
		<a class={class} href={href} title={title}>
			{children}
		</a>
	}
}
