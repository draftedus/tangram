use crate::{
	bar_chart::{BarChartOptions, BarChartSeries},
	box_chart::{BoxChartOptions, BoxChartSeries},
	chart::{Chart, ChartImpl},
	common::GridLineInterval,
	config::ChartConfig,
	feature_contributions_chart::{
		FeatureContributionsChartOptions, FeatureContributionsChartSeries,
	},
	line_chart::{LineChartOptions, LineChartSeries},
};
use html::{component, html, style};
use num_traits::ToPrimitive;
use wasm_bindgen::JsCast;
use web_sys::*;

pub fn hydrate_chart<T>(id: &str)
where
	T: ChartImpl,
	T::Options: serde::de::DeserializeOwned,
{
	let window = window().unwrap();
	let document = window.document().unwrap();
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

#[component]
pub fn BarChart(
	class: Option<String>,
	group_gap: Option<f64>,
	hide_legend: Option<bool>,
	id: Option<String>,
	series: Vec<BarChartSeries>,
	should_draw_x_axis_labels: Option<bool>,
	should_draw_y_axis_labels: Option<bool>,
	title: Option<String>,
	x_axis_title: Option<String>,
	y_axis_grid_line_interval: Option<GridLineInterval>,
	y_axis_title: Option<String>,
	y_max: Option<f64>,
	y_min: Option<f64>,
) {
	let options = BarChartOptions {
		group_gap,
		hide_legend,
		series,
		should_draw_x_axis_labels,
		should_draw_y_axis_labels,
		x_axis_title,
		y_axis_grid_line_interval,
		y_axis_title,
		y_max,
		y_min,
	};
	let hide_legend = hide_legend.unwrap_or(false);
	let container_style = style! {
		"padding-top" => "50%",
		"width" => "100%",
	};
	let legend_items: Vec<LegendItem> = options
		.series
		.iter()
		.filter_map(|series| {
			let title = if let Some(title) = &series.title {
				title
			} else {
				return None;
			};
			Some(LegendItem {
				color: series.color.clone(),
				title: title.clone(),
			})
		})
		.collect();
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{title}</ChartTitle>
			{if !hide_legend {
				Some(html! { <ChartLegend items={legend_items} /> })
			} else {
				None
			}}
			<div
				class={class}
				data-chart-type="bar"
				data-options={options}
				id={id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[component]
pub fn BoxChart(
	class: Option<String>,
	hide_legend: Option<bool>,
	id: Option<String>,
	series: Vec<BoxChartSeries>,
	should_draw_x_axis_labels: Option<bool>,
	should_draw_y_axis_labels: Option<bool>,
	title: Option<String>,
	x_axis_title: Option<String>,
	y_axis_title: Option<String>,
	y_max: Option<f64>,
	y_min: Option<f64>,
) {
	let options = BoxChartOptions {
		hide_legend,
		series,
		should_draw_x_axis_labels,
		should_draw_y_axis_labels,
		title: title.clone(),
		x_axis_title,
		y_axis_title,
		y_max,
		y_min,
	};
	let hide_legend = hide_legend.unwrap_or(false);
	let container_style = style! {
		"padding-top" => "50%",
		"width" => "100%",
	};
	let legend_items: Vec<LegendItem> = options
		.series
		.iter()
		.filter_map(|series| {
			let title = if let Some(title) = &series.title {
				title
			} else {
				return None;
			};
			Some(LegendItem {
				color: series.color.clone(),
				title: title.clone(),
			})
		})
		.collect();
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{title}</ChartTitle>
			{if !hide_legend {
				Some(html! { <ChartLegend items={legend_items} /> })
			} else {
				None
			}}
			<div
				class={class}
				data-chart-type="box"
				data-options={options}
				id={id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[component]
pub fn FeatureContributionsChart(
	class: Option<String>,
	id: Option<String>,
	include_x_axis_title: Option<bool>,
	include_y_axis_labels: Option<bool>,
	include_y_axis_title: Option<bool>,
	negative_color: String,
	positive_color: String,
	series: Vec<FeatureContributionsChartSeries>,
	title: Option<String>,
) {
	let chart_config = ChartConfig::default();
	let n_series = series.len();
	let options = FeatureContributionsChartOptions {
		include_x_axis_title,
		include_y_axis_labels,
		include_y_axis_title,
		negative_color,
		positive_color,
		series,
	};
	let inner_chart_height = n_series.to_f64().unwrap()
		* chart_config.feature_contributions_series_height
		+ (n_series - 1).to_f64().unwrap() * chart_config.feature_contributions_series_gap;
	let ChartConfig {
		bottom_padding,
		font_size,
		label_padding,
		top_padding,
		..
	} = chart_config;
	let height =
		inner_chart_height
			+ top_padding
			+ label_padding
			+ font_size + if include_x_axis_title.unwrap_or(false) {
			label_padding + font_size
		} else {
			0.0
		} + label_padding
			+ font_size + bottom_padding;
	let container_style = style! {
		"height" => format!("{}px", height),
		"width" => "100%",
	};
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{title}</ChartTitle>
			<div
				class={class}
				data-chart-type="feature_contributions"
				data-options={options}
				id={id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[component]
pub fn LineChart(
	class: Option<String>,
	hide_legend: Option<bool>,
	id: Option<String>,
	labels: Option<Vec<String>>,
	series: Vec<LineChartSeries>,
	should_draw_x_axis_labels: Option<bool>,
	should_draw_y_axis_labels: Option<bool>,
	title: Option<String>,
	x_axis_grid_line_interval: Option<GridLineInterval>,
	x_axis_title: Option<String>,
	x_max: Option<f64>,
	x_min: Option<f64>,
	y_axis_grid_line_interval: Option<GridLineInterval>,
	y_axis_title: Option<String>,
	y_max: Option<f64>,
	y_min: Option<f64>,
) {
	let options = LineChartOptions {
		hide_legend,
		labels,
		series,
		should_draw_x_axis_labels,
		should_draw_y_axis_labels,
		title: title.clone(),
		x_axis_grid_line_interval,
		x_axis_title,
		x_max,
		x_min,
		y_axis_grid_line_interval,
		y_axis_title,
		y_max,
		y_min,
	};
	let hide_legend = hide_legend.unwrap_or(false);
	let container_style = style! {
		"padding-top" => "50%",
		"width" => "100%",
	};
	let legend_items: Vec<LegendItem> = options
		.series
		.iter()
		.filter_map(|series| {
			let title = if let Some(title) = &series.title {
				title
			} else {
				return None;
			};
			Some(LegendItem {
				color: series.color.clone(),
				title: title.clone(),
			})
		})
		.collect();
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{title}</ChartTitle>
			{if !hide_legend {
				Some(html! { <ChartLegend items={legend_items} /> })
			} else {
				None
			}}
			<div
				class={class}
				data-chart-type="line"
				data-options={options}
				id={id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[component]
pub fn ChartTitle() {
	html! {
		<div class="chart-title">{children}</div>
	}
}

#[derive(Clone)]
pub struct LegendItem {
	color: String,
	title: String,
}

#[component]
pub fn ChartLegend(items: Vec<LegendItem>) {
	html! {
		<div class="chart-legend-wrapper">
			{items.into_iter().map(|item| html! {
				<ChartLegendItem
					color={item.color}
					title={item.title}
				/>
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[component]
fn ChartLegendItem(color: String, title: String) {
	let style = style! {
		"background-color" => color,
	};
	html! {
		<div class="chart-legend-item">
			<div class="chart-legend-indicator" style={style}></div>
			<div class="chart-legend-title">{title}</div>
		</div>
	}
}
