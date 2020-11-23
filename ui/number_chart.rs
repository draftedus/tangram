use html::{component, html};

#[component]
pub fn NumberChart(title: String, value: String) {
	html! {
		<div class="number-chart-wrapper">
			<div class="number-chart-value">{value}</div>
			<div class="number-chart-title">{title}</div>
		</div>
	}
}
