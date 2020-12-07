use crate::Token;
use html::{classes, component, html};
use wasm_bindgen::JsCast;

fn default_value_formatter(value: Option<f32>) -> String {
	match value {
		Some(value) => value.to_string(),
		None => "N/A".to_owned(),
	}
}

#[component]
pub fn NumberComparisonChart(
	id: Option<String>,
	color_a: Option<String>,
	color_b: Option<String>,
	title: Option<String>,
	value_a: Option<f32>,
	value_a_title: Option<String>,
	value_b: Option<f32>,
	value_b_title: String,
	value_formatter: Option<fn(f32) -> String>,
) {
	let difference_string = difference_string(value_a, value_b);
	let difference_class = difference_class(value_a, value_b);
	let difference_class = classes!("number-comparison-difference", difference_class);
	html! {
		<div class="number-comparison-wrapper" id={id}>
			<div class="number-comparison-title">{title}</div>
			<div class={difference_class} data-field={"difference"}>{difference_string}</div>
			<div class="number-comparison-inner-wrapper">
				<div class="number-comparison-value" data-field="value-a">
					{value_a.map(|value_a| value_a.to_string()).unwrap_or_else(|| "N/A".to_owned())}
				</div>
				<div class="number-comparison-value" data-field="value-b">
					{value_b.map(|value_b| value_b.to_string()).unwrap_or_else(|| "N/A".to_owned())}
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

pub fn difference_class(value_a: Option<f32>, value_b: Option<f32>) -> String {
	match (value_a, value_b) {
		(Some(value_a), Some(value_b)) => match value_a.partial_cmp(&value_b) {
			Some(ordering) => match ordering {
				std::cmp::Ordering::Less => "number-comparison-positive".to_owned(),
				std::cmp::Ordering::Equal => "number-comparison-equals".to_owned(),
				std::cmp::Ordering::Greater => "number-comparison-negative".to_owned(),
			},
			None => "".to_owned(),
		},
		(_, _) => "number-comparison-na".to_owned(),
	}
}

pub fn difference_string(value_a: Option<f32>, value_b: Option<f32>) -> String {
	match (value_a, value_b) {
		(Some(value_a), Some(value_b)) => match value_a.partial_cmp(&value_b) {
			Some(ordering) => match &ordering {
				std::cmp::Ordering::Less => {
					format!("+{}", default_value_formatter(Some(value_b - value_a)))
				}
				std::cmp::Ordering::Equal => "equal".to_owned(),
				std::cmp::Ordering::Greater => default_value_formatter(Some(value_b - value_a)),
			},
			None => "N/A".to_owned(),
		},
		(_, _) => "N/A".to_owned(),
	}
}

pub fn update_number_comparison_chart(id: &str, value_a: Option<f32>, value_b: Option<f32>) {
	let document = web_sys::window().unwrap().document().unwrap();
	let difference_element = document
		.query_selector(&format!("#{} [data-field='difference']", id))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	let value_a_element = document
		.query_selector(&format!("#{} [data-field='value-a']", id))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	let value_b_element = document
		.query_selector(&format!("#{} [data-field='value-b']", id))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	value_a_element.set_inner_html(&default_value_formatter(value_a));
	value_b_element.set_inner_html(&default_value_formatter(value_b));
	difference_element.set_inner_html(&difference_string(value_a, value_b));
	let difference_class = difference_class(value_a, value_b);
	let difference_class = classes!("number-comparison-difference", difference_class);
	difference_element.set_class_name(&difference_class);
}
