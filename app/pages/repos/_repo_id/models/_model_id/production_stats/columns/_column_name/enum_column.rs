use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	metrics_row::MetricsRow,
	time::overall_chart_title,
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_charts::bar_chart::{BarChartPoint, BarChartSeries};
use tangram_charts::components::BarChart;
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;

#[derive(Clone)]
pub struct EnumColumnProps {
	pub alert: Option<String>,
	pub absent_count: u64,
	pub column_name: String,
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub invalid_count: u64,
	pub overall_chart_data: Vec<(String, EnumOverallHistogramEntry)>,
	pub overall_invalid_chart_data: Option<Vec<(String, u64)>>,
	pub row_count: u64,
}

#[derive(Clone)]
pub struct EnumIntervalChartDataPoint {
	pub label: String,
	pub histogram: Vec<(String, u64)>,
}

#[derive(Clone)]
pub struct EnumOverallHistogramEntry {
	pub production_count: u64,
	pub production_fraction: f32,
	pub training_count: u64,
	pub training_fraction: f32,
}

#[component]
pub fn EnumColumn(props: EnumColumnProps) {
	let overall_chart_series = vec![
		BarChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: props
				.overall_chart_data
				.iter()
				.enumerate()
				.map(|(index, (label, value))| BarChartPoint {
					label: label.to_owned(),
					x: index.to_f64().unwrap(),
					y: Some(value.training_fraction.to_f64().unwrap()),
				})
				.collect(),
			title: Some("Training".to_owned()),
		},
		BarChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.overall_chart_data
				.iter()
				.enumerate()
				.map(|(index, (label, value))| BarChartPoint {
					label: label.to_owned(),
					x: index.to_f64().unwrap(),
					y: Some(value.production_fraction.to_f64().unwrap()),
				})
				.collect(),
			title: Some("Production".to_owned()),
		},
	];
	let overall_distribution_chart_title = overall_chart_title(
		props.date_window,
		format!("Distribution of Unique Values for {}", props.column_name),
	);
	html! {
		<ui::S2>
			{props.alert.map(|alert| html! {
				<ui::Alert
					title={None}
					level={ui::Level::Danger}
				>
					{alert}
				</ui::Alert>
			})}
			<ui::Card>
				<BarChart
					class={None}
					hide_legend={None}
					group_gap={None}
					id={"enum_overall".to_owned()}
					series={overall_chart_series}
					title={overall_distribution_chart_title}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					y_axis_grid_line_interval={None}
					x_axis_title={props.column_name}
					y_axis_title={"Percent".to_owned()}
					y_max={1.0}
					y_min={None}
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
			<ui::H2 center={false}>{"Unique Values"}</ui::H2>
			<ui::Table width={"100%".to_owned()}>
				<ui::TableHeader>
					<ui::TableRow color={None}>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Value"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Training Count"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Production Count"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Training Fraction"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Production Fraction"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
					{props.overall_chart_data.iter().map(|(label, entry)| html! {
						<ui::TableRow color={None}>
							<ui::TableCell color={None} expand={false}>
								{label.to_owned()}
							</ui::TableCell>
							<ui::TableCell color={None} expand={false}>
								{ui::format_number(entry.training_count)}
							</ui::TableCell>
							<ui::TableCell color={None} expand={false}>
								{ui::format_number(entry.production_count)}
							</ui::TableCell>
							<ui::TableCell color={None} expand={false}>
								{ui::format_percent(entry.training_fraction)}
							</ui::TableCell>
							<ui::TableCell color={None} expand={false}>
								{ui::format_percent(entry.production_fraction)}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
			{props.overall_invalid_chart_data.map(|overall_invalid_chart_data| html! {
				<>
					<ui::H2 center={false}>{"Invalid Values"}</ui::H2>
					<ui::Table width={"100%".to_owned()}>
						<ui::TableHeader>
							<ui::TableRow color={None}>
								<ui::TableHeaderCell
									color={None}
									expand={None}
									text_align={None}
								>
									{"Value"}
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
						{overall_invalid_chart_data.into_iter().map(|(label, count)| html! {
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
				</>
			})}
		</ui::S2>
	}
}
