use html::{classes, component, html};

#[component]
pub fn NestedNav() {
	html! {
		<div class="nested-nav">{children}</div>
	}
}

#[component]
pub fn NestedNavSection() {
	html! {
		<div class="nested-nav-section">{children}</div>
	}
}

#[component]
pub fn NestedNavSectionTitle() {
	html! {
		<div class="nested-nav-section-title">{children}</div>
	}
}

#[component]
pub fn NestedNavItem(href: String, selected: Option<bool>) {
	let class = classes!("nested_nav_item", selected.map(|_| "nested-nav-selected"));
	html! {
		<div class={class}>
			<a href={href}>{children}</a>
		</div>
	}
}
