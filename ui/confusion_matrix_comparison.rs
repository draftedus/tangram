use crate::Token;
use html::{classes, component, html, style};
use wasm_bindgen::JsCast;

// |-----------------------------------------------------------|
// |           ||       |                Actual                |
// |===========||==============================================|
// |           ||       |       True       |      False        |
// |           ||----------------------------------------------|
// |           ||       |                  |                   |
// |           || True  |  True Positives  |  False Positives  |
// |           ||       |                  |                   |
// | Predicted ||-------|--------------------------------------|
// |           ||       |                  |                   |
// |           || False |  False Negatives |  True Negatives   |
// |           ||       |                  |                   |
// |-----------------------------------------------------------|

#[derive(Clone)]
pub struct ConfusionMatrixComparisonValue {
	pub false_negative: f32,
	pub false_positive: f32,
	pub true_negative: f32,
	pub true_positive: f32,
}

fn default_value_formatter(value: f32) -> String {
	if value.is_finite() {
		value.to_string()
	} else {
		"N/A".to_owned()
	}
}

#[component]
pub fn ConfusionMatrixComparison(
	id: Option<String>,
	class_label: String,
	color_a: String,
	color_b: String,
	value_a: ConfusionMatrixComparisonValue,
	value_a_title: String,
	value_b: ConfusionMatrixComparisonValue,
	value_b_title: String,
) {
	html! {
		<div class="confusion-matrix-comparison-wrapper" id={id}>
			<ConfusionMatrixLabel area={"actual-true-label"} left={None}>
				<div>{"Actual"}</div>
				<Token color={None}>{class_label.clone()}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area={"actual-false-label"} left={None}>
				<div>{"Actual Not"}</div>
				<Token color={None}>{class_label.clone()}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area={"predicted-true-label"} left={Some(true)}>
				<div>{"Predicted"}</div>
				<Token color={None}>{class_label.clone()}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area={"predicted-false-label"} left={Some(true)}>
				<div>{"Predicted Not"}</div>
				<Token color={None}>{class_label}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixComparisonItem
				area={"true-positive"}
				color_a={color_a.clone()}
				color_b={color_b.clone()}
				label={"True Positives"}
				correct={true}
				value_a={value_a.true_positive}
				value_a_title={value_a_title.clone()}
				value_b={value_b.true_positive}
				value_b_title={value_b_title.clone()}
			/>
			<ConfusionMatrixComparisonItem
				area={"false-positive"}
				color_a={color_a.clone()}
				correct={false}
				color_b={color_b.clone()}
				label={"False Positives"}
				value_a={value_a.false_positive}
				value_a_title={value_a_title.clone()}
				value_b={value_b.false_positive}
				value_b_title={value_b_title.clone()}
			/>
			<ConfusionMatrixComparisonItem
				area={"false-negative"}
				color_a={color_a.clone()}
				color_b={color_b.clone()}
				correct={false}
				label={"False Negatives"}
				value_a={value_a.false_negative}
				value_a_title={value_a_title.clone()}
				value_b={value_b.false_negative}
				value_b_title={value_b_title.clone()}
			/>
			<ConfusionMatrixComparisonItem
				area={"true-negative"}
				color_a={color_a}
				color_b={color_b}
				label={"True Negatives"}
				correct={true}
				value_a={value_a.true_negative}
				value_a_title={value_a_title}
				value_b={value_b.true_negative}
				value_b_title={value_b_title}
			/>
		</div>
	}
}

#[component]
fn ConfusionMatrixComparisonItem(
	area: String,
	color_a: String,
	color_b: String,
	label: String,
	correct: bool,
	value_a: f32,
	value_a_title: String,
	value_b: f32,
	value_b_title: String,
) {
	let wrapper_style = style! {
		"grid-area" => area,
	};
	let class = if correct {
		"confusion-matrix-comparison-item-correct-wrapper"
	} else {
		"confusion-matrix-comparison-item-incorrect-wrapper"
	};
	let class = classes!("confusion-matrix-comparison-item-wrapper", class);
	let value_a = value_a.to_string();
	let value_b = value_b.to_string();
	html! {
		<div class={class} style={wrapper_style} data-area={area}>
			<div class="confusion-matrix-comparison-item-title">{label}</div>
			<div class="confusion-matrix-comparison-number-comparison-wrapper">
				<div class="confusion-matrix-comparison-item-value" data-field={"value-a"}>
					{value_a}
				</div>
				<div class="confusion-matrix-comparison-item-value" data-field={"value-b"}>
					{value_b}
				</div>
				<div>
					<Token color={Some(color_a)}>{value_a_title}</Token>
				</div>
				<div>
					<Token color={Some(color_b)}>{value_b_title}</Token>
				</div>
			</div>
		</div>
	}
}

#[component]
pub fn ConfusionMatrixLabel(area: String, left: Option<bool>) {
	let left = left.unwrap_or(false);
	let justify_items = if left { "end" } else { "center" };
	let style = style! {
		"grid-area" => area,
		"justify-items" => justify_items,
	};
	html! {
		<div class="confusion-matrix-comparison-label" style={style}>
			{children}
		</div>
	}
}

pub fn update_confusion_matrix_comparison_item(id: &str, area: &str, value_a: f32, value_b: f32) {
	let document = web_sys::window().unwrap().document().unwrap();
	let value_a_element = document
		.query_selector(&format!(
			"#{} [data-area='{}'] [data-field='value-a']",
			id, area
		))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	let value_b_element = document
		.query_selector(&format!(
			"#{} [data-area='{}'] [data-field='value-b']",
			id, area
		))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	value_a_element.set_inner_html(&default_value_formatter(value_a));
	value_b_element.set_inner_html(&default_value_formatter(value_b));
}

pub fn update_confusion_matrix_comparison(
	id: &str,
	value_a: ConfusionMatrixComparisonValue,
	value_b: ConfusionMatrixComparisonValue,
) {
	update_confusion_matrix_comparison_item(
		id,
		"false-positive",
		value_a.false_positive,
		value_b.false_positive,
	);
	update_confusion_matrix_comparison_item(
		id,
		"false-negative",
		value_a.false_negative,
		value_b.false_negative,
	);
	update_confusion_matrix_comparison_item(
		id,
		"true-positive",
		value_a.true_positive,
		value_b.true_positive,
	);
	update_confusion_matrix_comparison_item(
		id,
		"true-negative",
		value_a.true_negative,
		value_b.true_negative,
	);
}
