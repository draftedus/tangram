use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	metrics_row::MetricsRow,
	time::overall_chart_title,
	tokens::PRODUCTION_COLOR,
};
use tangram_charts::bar_chart::{BarChartPoint, BarChartSeries};
use tangram_charts::components::BarChart;
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;

#[derive(Clone)]
pub struct TextColumnProps {
	pub absent_count: u64,
	pub alert: Option<String>,
	pub column_name: String,
	pub date_window_interval: DateWindowInterval,
	pub date_window: DateWindow,
	pub invalid_count: u64,
	pub overall_token_histogram: Vec<(String, u64)>,
	pub row_count: u64,
}

#[component]
pub fn TextColumn(props: TextColumnProps) {
	let overall_chart_series = vec![BarChartSeries {
		color: PRODUCTION_COLOR.to_owned(),
		data: props
			.overall_token_histogram
			.iter()
			.enumerate()
			.map(|(index, (label, count))| BarChartPoint {
				label: label.to_owned(),
				x: index.to_f64().unwrap(),
				y: Some(count.to_f64().unwrap()),
			})
			.collect(),
		title: Some("Production".to_owned()),
	}];
	let overall_distribution_chart_title = overall_chart_title(
		props.date_window,
		format!("Distribution of Unique Values for {}", props.column_name),
	);
	html! {
		<ui::S2>
			{props.alert.map(|alert| html! {
				<ui::Alert
					level={ui::Level::Danger}
					title={None}
				>
					{alert}
				</ui::Alert>
			})}
			<ui::Card>
				<BarChart
					class={None}
					group_gap={None}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					y_max={None}
					y_min={None}
					hide_legend={None}
					y_axis_grid_line_interval={None}
					id={"text_overall".to_owned()}
					series={overall_chart_series}
					title={overall_distribution_chart_title}
					x_axis_title={props.column_name}
					y_axis_title={"Count".to_owned()}
				/>
			</ui::Card>
			<MetricsRow>
				<ui::Card>
					<ui::NumberChart
						title={"Row Count".to_owned()}
						value={props.row_count.to_string()}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberChart
						title={"Absent Count".to_owned()}
						value={props.absent_count.to_string()}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberChart
						title={"Invalid Count".to_owned()}
						value={props.invalid_count.to_string()}
					/>
				</ui::Card>
			</MetricsRow>
			<ui::H2 center={false}>{"Unique Tokens"}</ui::H2>
			<ui::Table width={"100%".to_owned()}>
				<ui::TableHeader>
					<ui::TableRow color={None}>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Token"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Count"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
					{props.overall_token_histogram.into_iter().map(|(label, count)| html! {
						<ui::TableRow color={None}>
							<ui::TableCell
								color={None}
								expand={None}
							>
								{label}
							</ui::TableCell>
							<ui::TableCell
								color={None}
								expand={None}
							>
								{ui::format_number(count)}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
		</ui::S2>
	}
}
