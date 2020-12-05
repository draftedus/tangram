use crate::util::format_percent;
use crate::Token;
use html::{classes, component, html, style};
use num_traits::ToPrimitive;

// |---------------------------------------------------------|
// |           ||     |                Actual                |
// |===========||============================================|
// |           ||     |       Pos        |       Neg         |
// |           ||--------------------------------------------|
// |           ||     |                  |                   |
// |           || Pos |  True Positives  |  False Positives  |
// |           ||     |                  |                   |
// | Predicted ||-----|--------------------------------------|
// |           ||     |                  |                   |
// |           || Neg |  False Negatives |  True Negatives   |
// |           ||     |                  |                   |
// |---------------------------------------------------------|

#[component]
pub fn ConfusionMatrix(
	class_label: String,
	false_negatives: usize,
	false_positives: usize,
	true_negatives: usize,
	true_positives: usize,
) {
	let total = true_positives + true_negatives + false_positives + false_negatives;
	html! {
		<div class="confusion-matrix-wrapper">
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
			<ConfusionMatrixItem
				area={"true-positive"}
				correct={true}
				title={"True Positives"}
				total={total}
				value={true_positives}
			/>
			<ConfusionMatrixItem
				area={"false-positive"}
				title={"False Positives"}
				correct={false}
				total={total}
				value={false_positives}
			/>
			<ConfusionMatrixItem
				area={"false-negative"}
				title={"False Negatives"}
				correct={false}
				total={total}
				value={false_negatives}
			/>
			<ConfusionMatrixItem
				area={"true-negative"}
				correct={true}
				title={"True Negatives"}
				total={total}
				value={true_negatives}
			/>
		</div>
	}
}

#[component]
fn ConfusionMatrixItem(area: String, correct: bool, title: String, total: usize, value: usize) {
	let item_wrapper_style = style! {
			"grid-area" => area,
	};
	let class = match correct {
		true => "confusion-matrix-item-correct-wrapper",
		false => "confusion-matrix-item-incorrect-wrapper",
	};
	let class = classes!("confusion-matrix-item-wrapper", class);
	html! {
		<div
			class={class}
			style={item_wrapper_style}
		>
			<div class="confusion-matrix-item-title">{title}</div>
			<div class="confusion-matrix-item-value">
				{value.to_string()}
			</div>
			<div class="confusion-matrix-item-percent">
				{format_percent(value.to_f32().unwrap() / total.to_f32().unwrap())}
			</div>
		</div>
	}
}

#[component]
pub fn ConfusionMatrixLabel(area: String, left: Option<bool>) {
	let left = left.unwrap_or(false);
	let justify_items = if left { "end" } else { "auto" };
	let style = style! {
		"grid-area" => area,
		"justify-items" => justify_items,
	};
	html! {
		<div class="confusion-matrix-label" style={style}>
			{children}
		</div>
	}
}
