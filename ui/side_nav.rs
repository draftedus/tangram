use html::{classes, component, html};

#[component]
pub fn SideNav() {
	html! {
		<div class="side-nav">
			{children}
		</div>
	}
}

#[component]
pub fn SideNavSection() {
	html! {
		<div class="side-nav-section">
			{children}
		</div>
	}
}

#[component]
pub fn SideNavTitle() {
	html! {
		<div class="side-nav-title">
			{children}
		</div>
	}
}

#[component]
pub fn SideNavItem(href: String, selected: Option<bool>) {
	let selected = selected.and_then(|selected| {
		if selected {
			Some("side-nav-item-selected")
		} else {
			None
		}
	});
	let class_name = classes!("side-nav-item", selected);
	html! {
		<a class={class_name} href={href}>
			{children}
		</a>
	}
}
