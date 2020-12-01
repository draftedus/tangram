use crate::Token;
use html::{classes, component, html, style};
use num_traits::ToPrimitive;

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
	false_negative: usize,
	false_positive: usize,
	true_negative: usize,
	true_positive: usize,
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
	class_label: String,
	color_a: String,
	color_b: String,
	value_a: ConfusionMatrixComparisonValue,
	value_a_title: String,
	value_b: ConfusionMatrixComparisonValue,
	value_b_title: String,
	value_formatter: Option<fn(f32) -> String>,
) {
	let value_formatter = value_formatter.unwrap_or(default_value_formatter);
	html! {
		<div class="confusion-matrix-comparison-wrapper">
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
				value_formatter={value_formatter}
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
				value_formatter={value_formatter}
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
				value_formatter={value_formatter}
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
				value_formatter={value_formatter}
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
	value_a: usize,
	value_a_title: String,
	value_b: usize,
	value_b_title: String,
	value_formatter: fn(f32) -> String,
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
	let value_a = value_formatter(value_a.to_f32().unwrap());
	let value_b = value_formatter(value_b.to_f32().unwrap());
	html! {
		<div class={class} style={wrapper_style}>
			<div class="confusion-matrix-comparison-item-title">{label}</div>
			<div class="confusion-matrix-comparison-number-comparison-wrapper">
				<div class="confusion-matrix-comparison-item-value">
					{value_a}
				</div>
				<div class="confusion-matrix-comparison-item-value">
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
