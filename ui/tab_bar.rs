use html::{classes, component, html};

#[component]
pub fn TabBar() {
	html! {
	  <div class="tab-bar">
		{children}
	  </div>
	}
}

#[component]
pub fn Tab(disabled: Option<bool>, selected: Option<bool>) {
	let selected = selected.and_then(|selected| {
		if selected {
			Some("tab-bar-tab-selected")
		} else {
			None
		}
	});
	let disabled = disabled.and_then(|disabled| {
		if disabled {
			Some("tab-bar-tab-disabled")
		} else {
			None
		}
	});
	let className = classes!("tab-bar-tab", selected, disabled,);
	html! {
		<div
			class={className}
		>
			{children}
		</div>
	}
}

#[component]
pub fn TabLink(disabled: Option<bool>, href: String, selected: Option<bool>) {
	html! {
		<Tab selected={selected} disabled={None}>
			<a class="tab-bar-tab-link" href={href}>
				{children}
			</a>
		</Tab>
	}
}
