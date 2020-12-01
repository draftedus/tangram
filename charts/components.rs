use crate::bar_chart::{BarChartOptions, BarChartSeries};
use crate::common::GridLineInterval;
use html::{component, html, style};

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
