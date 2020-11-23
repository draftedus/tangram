use crate::Token;
use html::{classes, component, html};

fn default_value_formatter(value: f32) -> String {
	if value.is_finite() {
		value.to_string()
	} else {
		"N/A".to_owned()
	}
}

#[component]
pub fn NumberComparisonChart(
	color_a: Option<String>,
	color_b: Option<String>,
	title: Option<String>,
	value_a: f32,
	value_a_title: Option<String>,
	value_b: f32,
	value_b_title: String,
	value_formatter: Option<fn(f32) -> String>,
) {
	let value_formatter = value_formatter.unwrap_or(default_value_formatter);
	let difference_string = match value_a.partial_cmp(&value_b) {
		Some(ordering) => match &ordering {
			std::cmp::Ordering::Less => value_formatter(value_b - value_a),
			std::cmp::Ordering::Equal => "equal".into(),
			std::cmp::Ordering::Greater => format!("+ {}", value_formatter(value_b - value_a)),
		},
		None => "N/A".to_owned(),
	};
	let difference_class = match value_a.partial_cmp(&value_b) {
		Some(ordering) => match ordering {
			std::cmp::Ordering::Less => "number-comparison-negative",
			std::cmp::Ordering::Equal => "number-comparison-equals",
			std::cmp::Ordering::Greater => "number-comparison-positive",
		},
		None => "",
	};
	let difference_class = classes!("number-comparison-difference", difference_class);
	html! {
		<div class="number-comparison-wrapper">
			<div class="number-comparison-title">{title}</div>
			<div class={difference_class}>{difference_string}</div>
			<div class="number-comparison-inner-wrapper">
				<div class="number-comparison-value">
					{value_a.to_string()}
				</div>
				<div class="number-comparison-value">
					{value_b.to_string()}
				</div>
				<div>
					<Token color={color_a}>{value_a_title}</Token>
				</div>
				<div>
					<Token color={color_b}>{value_b_title}</Token>
				</div>
			</div>
		</div>
	}
}
