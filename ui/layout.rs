use html::{classes, component, html};

#[component]
pub fn S1() {
	html! {
		<div class="s1">
			{children}
		</div>
	}
}

#[component]
pub fn S2() {
	html! {
		<div class="s2">
			{children}
		</div>
	}
}

#[component]
pub fn SpaceBetween() {
	html! {
		<div class="space-between">
			{children}
		</div>
	}
}

#[component]
pub fn H1(center: Option<bool>) {
	let center = center.and_then(|center| if center { Some("center") } else { None });
	let class = classes!(Some("h1"), center);
	html! {
		<h1 class={class}>
			{children}
		</h1>
	}
}

#[component]
pub fn H2(center: Option<bool>) {
	let center = center.and_then(|center| if center { Some("center") } else { None });
	let class = classes!(Some("h2"), center);
	html! {
		<h2 class={class}>
			{children}
		</h2>
	}
}

#[component]
pub fn P() {
	html! {
		<p class="p">
			{children}
		</p>
	}
}

#[component]
pub fn List() {
	html! {
		<ul class="list">
			{children}
		</ul>
	}
}

#[component]
pub fn OrderedList() {
	html! {
		<ol class="ordered-list">
			{children}
		</ol>
	}
}

#[component]
pub fn ListItem() {
	html! {
		<li>
			{children}
		</li>
	}
}
