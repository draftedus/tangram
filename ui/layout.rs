use html::{component, html};

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
		<div class="s2">{children}</div>
	}
}

#[component]
pub fn SpaceBetween() {
	html! {
		<div class="space-between">{children}</div>
	}
}

#[component]
pub fn H1(center: Option<bool>) {
	html! {
		<h1 class={
			if center.is_some() {
				"h1 center"
			} else {
				"h1"
		}}>
			{children}
		</h1>
	}
}

#[component]
pub fn H2(center: Option<bool>) {
	html! {
		<h2 class={
			if center.is_some() {
				"h2 center"
			} else {
				"h2"
		}}>
			{children}
		</h2>
	}
}

#[component]
pub fn P() {
	html! {
		<p class="p">{children}</p>
	}
}

#[component]
pub fn List() {
	html! {
		<ul class="list">{children}</ul>
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
		<li>{children}</li>
	}
}
