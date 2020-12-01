pub mod bar_chart;
mod box_chart;
mod chart;
mod common;
pub mod components;
mod config;
mod feature_contributions_chart;
mod line_chart;
mod tooltip;

use self::chart::{Chart, ChartImpl};
use wasm_bindgen::JsCast;
use web_sys::*;

pub fn hydrate_chart<T>(id: &str)
where
	T: ChartImpl,
	T::Options: serde::de::DeserializeOwned,
{
	let document = window().unwrap().document().unwrap();
	let container = document
		.get_element_by_id(id)
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let options = container.dataset().get("options").unwrap();
	let options = serde_json::from_str(&options).unwrap();
	let chart = Chart::<T>::new(container);
	chart.borrow_mut().draw(options);
	std::mem::forget(chart);
}
