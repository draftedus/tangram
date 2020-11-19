use crate::bar_chart::{BarChartData, BarChartOptions};
use crate::common::GridLineInterval;
use html::{component, html, style};

#[component]
pub fn BarChart(
	class: Option<String>,
	id: Option<String>,
	title: Option<String>,
	data: BarChartData,
	group_gap: Option<usize>,
	hide_legend: Option<bool>,
	should_draw_x_axis_labels: Option<bool>,
	should_draw_y_axis_labels: Option<bool>,
	x_axis_title: Option<String>,
	y_axis_grid_line_interval: Option<GridLineInterval>,
	y_axis_title: Option<String>,
	y_max: Option<f32>,
	y_min: Option<f32>,
) {
	let options = BarChartOptions {
		class: class.clone(),
		id: id.clone(),
		data,
		title: title.clone(),
		group_gap,
		hide_legend,
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
		"width" =>  "100%",
	};
	let legend_items = vec![
		LegendItem {
			title: "hello".into(),
			color: "var(--blue)".into(),
		},
		LegendItem {
			title: "poop".into(),
			color: "var(--green)".into(),
		},
	];
	let options_json = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{title}</ChartTitle>
			{
				if hide_legend {
					None
				} else {
					Some(html! { <ChartLegend items={legend_items} /> })
				}
			}
			<div
				class={class}
				data-chart-type="bar"
				data-options={options_json}
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
			{
				items.into_iter().map(|item|
				html! {
					<ChartLegendItemCell
						color={item.color}
						title={item.title}
					/>
				}
			).collect::<Vec<_>>()
		}
		</div>
	}
}

#[component]
fn ChartLegendItemCell(color: String, title: String) {
	let style = style! {
		"background-color" => color,
	};
	html! (
		<div class="chart-legend-item">
			<div class="chart-legend-indicator" style={style}></div>
			<div class="chart-legend-title">{title}</div>
		</div>
	)
}
