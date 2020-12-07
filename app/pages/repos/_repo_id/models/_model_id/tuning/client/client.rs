use num_traits::ToPrimitive;
use tangram_app_pages_repos_repo_id_models_model_id_tuning_common::ClientProps;
use tangram_ui as ui;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::console;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();
	let data = document
		.get_element_by_id("tuning-page")
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	let client_props = data.dataset().get("props").unwrap();
	let client_props: ClientProps = serde_json::from_str(&client_props).unwrap();
	let thresholds = client_props
		.threshold_metrics
		.iter()
		.map(|metric| metric.threshold)
		.collect::<Vec<f32>>();
	let value_formatter: Box<dyn Fn(usize) -> String> =
		Box::new(move |value: usize| thresholds[value].to_string());
	ui::boot_slider("tuning-slider".to_owned(), Some(value_formatter));
	let document = web_sys::window().unwrap().document().unwrap();
	let slider = document.get_element_by_id("tuning-slider").unwrap();
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: web_sys::Event| {
		if let Some(current_target) = event.current_target() {
			let current_target = &current_target
				.dyn_into::<web_sys::HtmlInputElement>()
				.unwrap();
			let value: usize = current_target.value().parse().unwrap();
			let threshold_metrics = &client_props.threshold_metrics[value];
			let baseline_metrics = &client_props.baseline_metrics;
			ui::update_number_comparison_chart(
				"tuning-accuracy",
				baseline_metrics
					.accuracy
					.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.accuracy
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_number_comparison_chart(
				"tuning-precision",
				baseline_metrics
					.precision
					.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.precision
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_number_comparison_chart(
				"tuning-recall",
				baseline_metrics.recall.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.recall
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_number_comparison_chart(
				"tuning-f1-score",
				baseline_metrics
					.f1_score
					.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.f1_score
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_confusion_matrix_comparison(
				"tuning-confusion-matrix-comparison",
				ui::ConfusionMatrixComparisonValue {
					true_positive: baseline_metrics.true_positives.to_f32().unwrap(),
					false_positive: baseline_metrics.false_positives.to_f32().unwrap(),
					true_negative: baseline_metrics.true_negatives.to_f32().unwrap(),
					false_negative: baseline_metrics.false_negatives.to_f32().unwrap(),
				},
				ui::ConfusionMatrixComparisonValue {
					true_positive: threshold_metrics.true_positives.to_f32().unwrap(),
					false_positive: threshold_metrics.false_positives.to_f32().unwrap(),
					true_negative: threshold_metrics.true_negatives.to_f32().unwrap(),
					false_negative: threshold_metrics.false_negatives.to_f32().unwrap(),
				},
			);
		}
	}));
	if let Some(slider) = slider.dyn_ref::<web_sys::HtmlInputElement>() {
		slider
			.add_event_listener_with_callback("input", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
