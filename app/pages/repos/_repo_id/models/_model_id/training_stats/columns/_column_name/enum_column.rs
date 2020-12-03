use super::page::EnumProps;
use tangram_app_common::metrics_row::MetricsRow;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_deps::html::{self, component, html};
use tangram_deps::num_traits::ToPrimitive;
use tangram_ui as ui;

#[component]
pub fn EnumColumn(props: EnumProps) {
	let name = props.name;
	html! {
		<ui::S1>
			<ui::H1 center={None}>{name.clone()}</ui::H1>
			<ui::S2>
				<MetricsRow>
					<ui::Card>
						<ui::NumberChart
							title="Unique Count"
							value={props.unique_count.to_string()}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="Invalid Count"
							value={props.invalid_count.to_string()}
						/>
					</ui::Card>
				</MetricsRow>
				{props.histogram.map(|histogram| html! {
					<EnumColumnHistogram name={name} histogram />
				})}
			</ui::S2>
		</ui::S1>
	}
}

#[component]
fn EnumColumnHistogram(name: String, histogram: Vec<(String, u64)>) {
	let data = histogram
		.iter()
		.enumerate()
		.map(|(i, (value, count))| BarChartPoint {
			label: value.clone(),
			x: i.to_f64().unwrap(),
			y: Some(count.to_f64().unwrap()),
		})
		.collect();
	let histogram_chart_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data,
		title: Some("Unique Values".to_owned()),
	}];
	html! {
		<>
			<ui::Card>
				<BarChart
					class={None}
					group_gap={None}
					hide_legend={true}
					id={"enum_histogram".to_owned()}
					series={histogram_chart_series}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					title={format!("Histogram of Unique Values for {}", name)}
					x_axis_title={name}
					y_axis_grid_line_interval={None}
					y_axis_title={"Count".to_owned()}
					y_max={None}
					y_min={0.0}
				/>
			</ui::Card>
			<ui::Table width={Some("100%".to_owned())}>
				<ui::TableHeader>
					<ui::TableRow color={None}>
						<ui::TableHeaderCell color={None} expand={None} text_align={None}>
							{"Value"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell color={None} expand={None} text_align={None}>
							{"Count"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
					{histogram.iter().map(|(value, count)| html! {
						<ui::TableRow color={None}>
							<ui::TableCell color={None} expand={None}>
								{value.clone()}
							</ui::TableCell>
							<ui::TableCell color={None} expand={None}>
								{count.to_string()}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
		</>
	}
}
