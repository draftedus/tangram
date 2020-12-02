use super::page::NumberProps;
use html::{component, html};
use num_traits::ToPrimitive;
use tangram_app_common::metrics_row::MetricsRow;
use tangram_charts::{
	box_chart::BoxChartPoint,
	box_chart::{BoxChartSeries, BoxChartValue},
	components::BoxChart,
};
use tangram_ui as ui;

#[component]
pub fn NumberColumn(props: NumberProps) {
	let quantiles_chart_series = vec![BoxChartSeries {
		color: ui::colors::BLUE.to_string(),
		data: vec![BoxChartPoint {
			label: props.name.clone(),
			x: 0.0,
			y: Some(BoxChartValue {
				max: props.max.to_f64().unwrap(),
				min: props.min.to_f64().unwrap(),
				p25: props.p25.to_f64().unwrap(),
				p50: props.p50.to_f64().unwrap(),
				p75: props.p75.to_f64().unwrap(),
			}),
		}],
		title: Some("quartiles".to_owned()),
	}];
	html! {
		<ui::S1>
			<ui::H1 center={None}>{props.name.clone()}</ui::H1>
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
				<MetricsRow>
					<ui::Card>
						<ui::NumberChart
							title="Mean"
							value={ui::format_number(props.mean)}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="Standard Deviation"
							value={ui::format_number(props.std)}
						/>
					</ui::Card>
				</MetricsRow>
				<MetricsRow>
					<ui::Card>
						<ui::NumberChart
							title="Min"
							value={ui::format_number(props.min)}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="p25"
							value={ui::format_number(props.p25)}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="p50 (median)"
							value={ui::format_number(props.p50)}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="p75"
							value={ui::format_number(props.p75)}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="Max"
							value={ui::format_number(props.max)}
						/>
					</ui::Card>
				</MetricsRow>
				<ui::Card>
				<BoxChart
					class={None}
					hide_legend={None}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					x_axis_title={None}
					y_axis_title={None}
					y_max={None}
					y_min={None}
					id={"number_quantiles".to_owned()}
					series={quantiles_chart_series}
					title={format!("Distribution of Values for {}", props.name)}
				/>
				</ui::Card>
			</ui::S2>
		</ui::S1>
	}
}
