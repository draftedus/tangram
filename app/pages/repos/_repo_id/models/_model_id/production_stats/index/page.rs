use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	time::{interval_chart_title, overall_chart_title},
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken},
};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::{BarChart, BoxChart},
};
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;
use tangram_util::zip;

#[derive(Clone)]
pub struct Props {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub model_id: String,
	pub overall_column_stats_table: Vec<OverallColumnStats>,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: PredictionStatsChart,
	pub prediction_stats_interval_chart: PredictionStatsIntervalChart,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone)]
pub struct OverallColumnStats {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
	pub name: String,
	pub column_type: ColumnType,
}

#[derive(Clone)]
pub enum PredictionStatsChart {
	Regression(RegressionChartEntry),
	BinaryClassification(ClassificationChartEntry),
	MulticlassClassification(ClassificationChartEntry),
}

#[derive(Clone)]
pub enum PredictionStatsIntervalChart {
	Regression(Vec<RegressionChartEntry>),
	BinaryClassification(Vec<ClassificationChartEntry>),
	MulticlassClassification(Vec<ClassificationChartEntry>),
}

#[derive(Clone)]
pub enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

#[derive(Clone)]
pub struct PredictionCountChartEntry {
	pub count: u64,
	pub label: String,
}

#[derive(Clone)]
pub struct RegressionChartEntry {
	pub label: String,
	pub quantiles: ProductionTrainingQuantiles,
}

#[derive(Clone)]
pub struct ClassificationChartEntry {
	pub label: String,
	pub histogram: ProductionTrainingHistogram,
}

#[derive(Clone)]
pub struct ProductionTrainingHistogram {
	pub production: Vec<(String, u64)>,
	pub training: Vec<(String, u64)>,
}

#[derive(Clone)]
pub struct ProductionTrainingQuantiles {
	pub production: Option<Quantiles>,
	pub training: Quantiles,
}

#[derive(Clone)]
pub struct Quantiles {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let prediction_count_chart_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: props
			.prediction_count_chart
			.into_iter()
			.enumerate()
			.map(|(index, entry)| BarChartPoint {
				label: entry.label,
				x: index.to_f64().unwrap(),
				y: Some(entry.count.to_f64().unwrap()),
			})
			.collect::<Vec<_>>(),
		title: Some("Prediction Count".to_owned()),
	}];
	let prediction_count_title =
		interval_chart_title(&props.date_window_interval, "Prediction Count".to_owned());
	let interval_chart = match props.prediction_stats_interval_chart {
		PredictionStatsIntervalChart::Regression(data) => html! {
				<RegressionProductionStatsIntervalChart
					chart_data={data}
					date_window_interval={props.date_window_interval}
				/>
		},
		PredictionStatsIntervalChart::BinaryClassification(data) => {
			html! {
				<ClassificationProductionStatsIntervalChart
					chart_data={data}
					date_window_interval={props.date_window_interval}
				/>
			}
		}
		PredictionStatsIntervalChart::MulticlassClassification(data) => {
			html! {
				<ClassificationProductionStatsIntervalChart
					chart_data={data}
					date_window_interval={props.date_window_interval}
				/>
			}
		}
	};
	let production_stats_chart = match props.prediction_stats_chart {
		PredictionStatsChart::Regression(data) => html! {
			<RegressionProductionStatsChart
				chart_data={data}
				date_window={props.date_window.clone()}
			/>
		},
		PredictionStatsChart::BinaryClassification(data) => html! {
			<ClassificationProductionStatsChart
				chart_data={data}
				date_window={props.date_window.clone()}
			/>
		},
		PredictionStatsChart::MulticlassClassification(data) => html! {
			<ClassificationProductionStatsChart
				chart_data={data}
				date_window={props.date_window.clone()}
			/>
		},
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::ProductionStats}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Production Stats"}</ui::H1>
				<ui::Form
					action={None}
					autocomplete={None}
					post={None}
					enc_type={None}
					id={None}
				>
					<DateWindowSelectField date_window={props.date_window} />
					<noscript>
						<ui::Button
							color={None}
							disabled={None}
							href={None}
							id={None}
							download={None}
							button_type={ui::ButtonType::Submit}
						>
							{"Submit"}
						</ui::Button>
					</noscript>
				</ui::Form>
				<ui::Card>
					{interval_chart}
				</ui::Card>
				<ui::Card>
					<BarChart
						class={None}
						hide_legend={None}
						x_axis_title={None}
						group_gap={None}
						should_draw_x_axis_labels={None}
						should_draw_y_axis_labels={None}
						y_max={None}
						y_min={None}
						y_axis_title={None}
						y_axis_grid_line_interval={None}
						id={"prediction_count".to_owned()}
						series={prediction_count_chart_series}
						title={prediction_count_title}
					/>
				</ui::Card>
				<ui::Card>
					{production_stats_chart}
				</ui::Card>
				<ui::Table width={"100%".to_owned()}>
					<ui::TableHeader>
						<ui::TableRow color={None}>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Status"}
							</ui::TableHeaderCell>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Column"}
							</ui::TableHeaderCell>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Type"}
							</ui::TableHeaderCell>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Absent Count"}
							</ui::TableHeaderCell>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Invalid Count"}
							</ui::TableHeaderCell>
						</ui::TableRow>
					</ui::TableHeader>
					<ui::TableBody>
					{props.overall_column_stats_table.into_iter().map(|column| html! {
						<ui::TableRow color={None}>
							<ui::TableCell color={None} expand={None}>
								{if column.alert.is_some() {
									html! {
										<ui::AlertIcon
											alert={column.alert.unwrap()}
											level={ui::Level::Danger}
										>
											{"!"}
										</ui::AlertIcon>
									}
									} else {
										html! {
											<ui::AlertIcon
												alert={"All good".to_owned()}
												level={ui::Level::Success}
											>
												{"✓"}
											</ui::AlertIcon>
										}
								}}
							</ui::TableCell>
							<ui::TableCell color={None} expand={None}>
								<ui::Link
									href={format!("./columns/{}", column.name)}
									class={None}
									title={None}
								>
									{column.name}
								</ui::Link>
							</ui::TableCell>
							<ui::TableCell color={None} expand={None}>
								{column_type_token(&column.column_type)}
							</ui::TableCell>
							<ui::TableCell color={None} expand={None}>
								{column.absent_count.to_string()}
							</ui::TableCell>
							<ui::TableCell color={None} expand={None}>
								{column.invalid_count.to_string()}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
					</ui::TableBody>
				</ui::Table>
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}

#[component]
fn RegressionProductionStatsChart(chart_data: RegressionChartEntry, date_window: DateWindow) {
	let series = vec![
		BoxChartSeries {
			color: ui::colors::GREEN.to_owned(),
			data: vec![BoxChartPoint {
				label: chart_data.label.clone(),
				x: 0.0,
				y: Some(BoxChartValue {
					max: chart_data.quantiles.training.max.to_f64().unwrap(),
					min: chart_data.quantiles.training.min.to_f64().unwrap(),
					p25: chart_data.quantiles.training.p25.to_f64().unwrap(),
					p50: chart_data.quantiles.training.p50.to_f64().unwrap(),
					p75: chart_data.quantiles.training.p75.to_f64().unwrap(),
				}),
			}],
			title: Some("training quantiles".to_owned()),
		},
		BoxChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: vec![BoxChartPoint {
				label: chart_data.label,
				x: 0.0,
				y: chart_data
					.quantiles
					.production
					.map(|production| BoxChartValue {
						max: production.max.to_f64().unwrap(),
						min: production.min.to_f64().unwrap(),
						p25: production.p25.to_f64().unwrap(),
						p50: production.p50.to_f64().unwrap(),
						p75: production.p75.to_f64().unwrap(),
					}),
			}],
			title: Some("training quantiles".to_owned()),
		},
	];
	let title = overall_chart_title(&date_window, "Prediction Distribution Stats".to_owned());
	html! {
		<BoxChart
			class={None}
			hide_legend={None}
			should_draw_x_axis_labels={None}
			should_draw_y_axis_labels={None}
			x_axis_title={None}
			y_axis_title={None}
			y_min={None}
			y_max={None}
			id={"quantiles_overall".to_owned()}
			series={series}
			title={title}
		/>
	}
}

#[component]
fn RegressionProductionStatsIntervalChart(
	chart_data: Vec<RegressionChartEntry>,
	date_window_interval: DateWindowInterval,
) {
	let series = vec![BoxChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: chart_data
			.into_iter()
			.enumerate()
			.map(|(index, entry)| BoxChartPoint {
				label: entry.label,
				x: index.to_f64().unwrap(),
				y: entry.quantiles.production.map(|production| BoxChartValue {
					max: production.max.to_f64().unwrap(),
					min: production.min.to_f64().unwrap(),
					p25: production.p25.to_f64().unwrap(),
					p50: production.p50.to_f64().unwrap(),
					p75: production.p75.to_f64().unwrap(),
				}),
			})
			.collect::<Vec<_>>(),
		title: Some("quantiles".to_owned()),
	}];
	let title = interval_chart_title(
		&date_window_interval,
		"Prediction Distribution Stats".to_owned(),
	);
	html! {
		<BoxChart
			class={None}
			hide_legend={None}
			should_draw_x_axis_labels={None}
			should_draw_y_axis_labels={None}
			x_axis_title={None}
			y_axis_title={None}
			y_min={None}
			y_max={None}
			id={"quantiles_intervals".to_owned()}
			series={series}
			title={title}
		/>
	}
}

#[component]
fn ClassificationProductionStatsChart(
	chart_data: ClassificationChartEntry,
	date_window: DateWindow,
) {
	let color_options = vec![
		ui::colors::GREEN,
		ui::colors::BLUE,
		ui::colors::INDIGO,
		ui::colors::PURPLE,
		ui::colors::PINK,
		ui::colors::RED,
		ui::colors::ORANGE,
		ui::colors::YELLOW,
	];
	let classes = chart_data
		.histogram
		.production
		.iter()
		.cloned()
		.map(|(class, _)| class)
		.collect::<Vec<_>>();
	let title = overall_chart_title(&date_window, "Prediction Stats".to_owned());
	let series = zip!(classes.iter(), chart_data.histogram.production.iter())
		.enumerate()
		.map(|(index, (class, entry))| {
			let color = color_options[index % color_options.len()].to_owned();
			BarChartSeries {
				color,
				data: vec![BarChartPoint {
					label: chart_data.label.to_owned(),
					x: 0.0,
					y: Some(entry.1.to_f64().unwrap()),
				}],
				title: Some(class.to_owned()),
			}
		})
		.collect::<Vec<_>>();
	html! {
		<BarChart
			class={None}
			hide_legend={None}
			x_axis_title={None}
			group_gap={None}
			should_draw_x_axis_labels={None}
			should_draw_y_axis_labels={None}
			y_max={None}
			y_min={None}
			y_axis_title={None}
			y_axis_grid_line_interval={None}
			id={"histogram_overall".to_owned()}
			series={series}
			title={title}
		/>
	}
}

#[component]
fn ClassificationProductionStatsIntervalChart(
	chart_data: Vec<ClassificationChartEntry>,
	date_window_interval: DateWindowInterval,
) {
	let color_options = vec![
		ui::colors::GREEN,
		ui::colors::BLUE,
		ui::colors::INDIGO,
		ui::colors::PURPLE,
		ui::colors::PINK,
		ui::colors::RED,
		ui::colors::ORANGE,
		ui::colors::YELLOW,
	];
	let title = interval_chart_title(&date_window_interval, "Prediction Stats".to_owned());
	let classes = chart_data[0]
		.histogram
		.production
		.iter()
		.cloned()
		.map(|(class, _)| class)
		.collect::<Vec<_>>();
	let series = classes
		.iter()
		.enumerate()
		.map(|(index, class)| {
			let color = color_options[index % color_options.len()].to_owned();
			BarChartSeries {
				color,
				data: chart_data
					.iter()
					.enumerate()
					.map(|(entry_index, entry)| BarChartPoint {
						label: entry.label.to_owned(),
						x: entry_index.to_f64().unwrap(),
						y: Some(entry.histogram.production[index].1.to_f64().unwrap()),
					})
					.collect::<Vec<_>>(),
				title: Some(class.to_owned()),
			}
		})
		.collect::<Vec<_>>();
	html! {
		<BarChart
			class={None}
			hide_legend={None}
			x_axis_title={None}
			group_gap={None}
			should_draw_x_axis_labels={None}
			should_draw_y_axis_labels={None}
			y_max={None}
			y_min={None}
			y_axis_title={None}
			y_axis_grid_line_interval={None}
			id={"histogram_intervals".to_owned()}
			series={series}
			title={title}
		/>
	}
}

fn column_type_token(column_type: &ColumnType) -> html::Node {
	match column_type {
		ColumnType::Unknown => html! {
			<UnknownColumnToken />
		},
		ColumnType::Number => html! {
			<NumberColumnToken />
		},
		ColumnType::Enum => html! {
			<EnumColumnToken />
		},
		ColumnType::Text => html! {
			<TextColumnToken />
		},
	}
}
