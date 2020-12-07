use crate::tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken};
use std::convert::TryInto;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::{BarChart, FeatureContributionsChart},
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
	serde_json,
};
use tangram_ui as ui;

#[derive(Clone)]
pub struct PredictionResultProps {
	pub input_table: InputTable,
	pub prediction: Prediction,
}

#[derive(Clone)]
pub struct InputTable {
	pub rows: Vec<InputTableRow>,
}

#[derive(Clone)]
pub struct InputTableRow {
	pub column_name: String,
	pub column_type: ColumnType,
	pub value: serde_json::Value,
}

#[derive(Clone)]
pub enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

#[derive(Clone)]
pub enum Prediction {
	Regression(RegressionPredictionResultProps),
	BinaryClassification(BinaryClassificationPredictionResultProps),
	MulticlassClassification(MulticlassClassificationPredictionResultProps),
}

#[derive(Clone)]
pub struct RegressionPredictionResultProps {
	value: f32,
	feature_contributions_chart_series: Vec<FeatureContributionsChartSeriesData>,
}

#[derive(Clone)]
pub struct BinaryClassificationPredictionResultProps {
	class_name: String,
	probability: f32,
	feature_contributions_chart_series: Vec<FeatureContributionsChartSeriesData>,
}

#[derive(Clone)]
pub struct MulticlassClassificationPredictionResultProps {
	class_name: String,
	probability: f32,
	probabilities: Vec<(String, f32)>,
	feature_contributions_chart_series: Vec<FeatureContributionsChartSeriesData>,
}

#[derive(Clone)]
pub struct FeatureContributionsChartSeriesData {
	baseline: f32,
	baseline_label: String,
	label: String,
	output: f32,
	output_label: String,
	values: Vec<FeatureContributionsChartValueData>,
}

#[derive(Clone)]
pub struct FeatureContributionsChartValueData {
	feature: String,
	value: f32,
}

pub fn predict(
	model: tangram_core::model::Model,
	example: serde_json::Map<String, serde_json::Value>,
) -> Prediction {
	let predict_model: tangram_core::predict::Model = model.try_into().unwrap();
	let examples = tangram_core::predict::PredictInput(vec![example]);
	let output = tangram_core::predict::predict(&predict_model, examples, None);
	let predict_output: Prediction = match output {
		tangram_core::predict::PredictOutput::Regression(mut output) => {
			let output = output.remove(0);
			let feature_contributions = output.feature_contributions.unwrap();
			let feature_contributions_chart_data = vec![FeatureContributionsChartSeriesData {
				baseline: feature_contributions.baseline_value,
				baseline_label: format!("{}", feature_contributions.baseline_value),
				label: "output".to_owned(),
				output: feature_contributions.output_value,
				output_label: format!("{}", feature_contributions.output_value),
				values: feature_contributions
					.feature_contributions
					.into_iter()
					.map(compute_feature_contributions_chart_value)
					.collect(),
			}];
			let prediction = RegressionPredictionResultProps {
				feature_contributions_chart_series: feature_contributions_chart_data,
				value: output.value,
			};
			Prediction::Regression(prediction)
		}
		tangram_core::predict::PredictOutput::BinaryClassification(mut output) => {
			let output = output.remove(0);
			let feature_contributions = output.feature_contributions.unwrap();
			let feature_contributions_chart_data = vec![FeatureContributionsChartSeriesData {
				baseline: feature_contributions.baseline_value,
				baseline_label: format!("{}", feature_contributions.baseline_value),
				label: "output".to_owned(),
				output: feature_contributions.output_value,
				output_label: format!("{}", feature_contributions.output_value),
				values: feature_contributions
					.feature_contributions
					.into_iter()
					.map(compute_feature_contributions_chart_value)
					.collect(),
			}];
			let prediction = BinaryClassificationPredictionResultProps {
				class_name: output.class_name,
				probability: output.probability,
				feature_contributions_chart_series: feature_contributions_chart_data,
			};
			Prediction::BinaryClassification(prediction)
		}
		tangram_core::predict::PredictOutput::MulticlassClassification(mut output) => {
			let output = output.remove(0);
			let feature_contributions_chart_data: Vec<FeatureContributionsChartSeriesData> = output
				.feature_contributions
				.unwrap()
				.into_iter()
				.map(
					|(class, feature_contributions)| FeatureContributionsChartSeriesData {
						baseline: feature_contributions.baseline_value,
						baseline_label: format!("{}", feature_contributions.baseline_value),
						label: class,
						output: feature_contributions.output_value,
						output_label: format!("{}", feature_contributions.output_value),
						values: feature_contributions
							.feature_contributions
							.into_iter()
							.map(compute_feature_contributions_chart_value)
							.collect(),
					},
				)
				.collect();
			let prediction = MulticlassClassificationPredictionResultProps {
				class_name: output.class_name,
				probability: output.probability,
				probabilities: output.probabilities.into_iter().collect::<Vec<_>>(),
				feature_contributions_chart_series: feature_contributions_chart_data,
			};
			Prediction::MulticlassClassification(prediction)
		}
	};
	predict_output
}

fn compute_feature_contributions_chart_value(
	feature_contribution: tangram_core::predict::FeatureContribution,
) -> FeatureContributionsChartValueData {
	match feature_contribution {
		tangram_core::predict::FeatureContribution::Identity {
			column_name,
			feature_contribution_value,
		} => FeatureContributionsChartValueData {
			feature: column_name,
			value: feature_contribution_value,
		},
		tangram_core::predict::FeatureContribution::Normalized {
			column_name,
			feature_contribution_value,
		} => FeatureContributionsChartValueData {
			feature: column_name,
			value: feature_contribution_value,
		},
		tangram_core::predict::FeatureContribution::OneHotEncoded {
			column_name,
			option,
			feature_value,
			feature_contribution_value,
		} => {
			let predicate = if feature_value { "is" } else { "is not" };
			let option = option
				.map(|option| format!("\"{}\"", option))
				.unwrap_or_else(|| "invalid".to_owned());
			let feature = format!("{} {} {}", column_name, predicate, option);
			FeatureContributionsChartValueData {
				feature,
				value: feature_contribution_value,
			}
		}
		tangram_core::predict::FeatureContribution::BagOfWords {
			column_name,
			token,
			feature_value,
			feature_contribution_value,
		} => {
			let predicate = if feature_value {
				"contains"
			} else {
				"does not contain"
			};
			let feature = format!("{} {} \"{}\"", column_name, predicate, token);
			FeatureContributionsChartValueData {
				feature,
				value: feature_contribution_value,
			}
		}
	}
}

#[component]
pub fn PredictionResult(props: PredictionResultProps) {
	let inner = match props.prediction {
		Prediction::Regression(inner) => {
			html! { <RegressionPrediction props={inner} /> }
		}
		Prediction::BinaryClassification(inner) => {
			html! { <BinaryClassificationPrediction props={inner} /> }
		}
		Prediction::MulticlassClassification(inner) => {
			html! { <MulticlassClassificationPrediction props={inner} /> }
		}
	};
	html! {
		<ui::S2>
			<ui::H2 center={false}>{"Input"}</ui::H2>
			<ui::Table width={"100%".to_owned()}>
				<ui::TableHeader>
					<ui::TableRow color={None}>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Column Name"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Column Type"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							color={None}
							expand={None}
							text_align={None}
						>
							{"Value"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
				{props.input_table.rows.iter().map(|input_table_row| {
					html! {
					<ui::TableRow color={None}>
						<ui::TableCell
							color={None}
							expand={None}
						>
							{input_table_row.column_name.to_owned()}
						</ui::TableCell>
						<ui::TableCell
							color={None}
							expand={None}
						>
							{column_type_token(&input_table_row.column_type)}
						</ui::TableCell>
						<ui::TableCell
							color={None}
							expand={None}
						>
							{input_table_row.value.to_string()}
						</ui::TableCell>
					</ui::TableRow>
					}
				}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
			{inner}
		</ui::S2>
	}
}

fn column_type_token(column_type: &ColumnType) -> html::Node {
	match column_type {
		ColumnType::Unknown => {
			html! { <UnknownColumnToken /> }
		}
		ColumnType::Number => {
			html! { <NumberColumnToken /> }
		}
		ColumnType::Enum => {
			html! { <EnumColumnToken /> }
		}
		ColumnType::Text => {
			html! { <TextColumnToken /> }
		}
	}
}

#[component]
pub fn RegressionPrediction(props: RegressionPredictionResultProps) {
	let series = props
		.feature_contributions_chart_series
		.iter()
		.map(|feature_contribution| FeatureContributionsChartSeries {
			title: feature_contribution.label.clone(),
			baseline: feature_contribution.baseline.to_f64().unwrap(),
			baseline_label: feature_contribution.baseline_label.to_owned(),
			output: feature_contribution.output.to_f64().unwrap(),
			output_label: feature_contribution.output_label.to_owned(),
			values: feature_contribution
				.values
				.iter()
				.map(|value| FeatureContributionsChartValue {
					feature: value.feature.to_owned(),
					value: value.value.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
		})
		.collect::<Vec<_>>();
	html! {
		<ui::S2>
			<ui::H2 center={false}>{"Output"}</ui::H2>
			<ui::Card>
				<ui::NumberChart
					title={"Prediction".to_owned()}
					value={props.value.to_string()}
				/>
			</ui::Card>
			<ui::H2 center={false}>{"Explanation"}</ui::H2>
			<ui::P>
				{"This chart shows how the input values influenced the model's output."}
			</ui::P>
			<ui::Card>
				<FeatureContributionsChart
					class={None}
					title={None}
					id={"regression_feature_contributions".to_owned()}
					include_x_axis_title={true}
					include_y_axis_labels={false}
					include_y_axis_title={false}
					negative_color={ui::colors::RED.to_owned()}
					positive_color={ui::colors::GREEN.to_owned()}
					series={series}
				/>
			</ui::Card>
		</ui::S2>
	}
}

#[component]
pub fn BinaryClassificationPrediction(props: BinaryClassificationPredictionResultProps) {
	let series = props
		.feature_contributions_chart_series
		.iter()
		.map(|feature_contribution| FeatureContributionsChartSeries {
			title: feature_contribution.label.clone(),
			baseline: feature_contribution.baseline.to_f64().unwrap(),
			baseline_label: feature_contribution.baseline_label.to_owned(),
			output: feature_contribution.output.to_f64().unwrap(),
			output_label: feature_contribution.output_label.to_owned(),
			values: feature_contribution
				.values
				.iter()
				.map(|value| FeatureContributionsChartValue {
					feature: value.feature.to_owned(),
					value: value.value.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
		})
		.collect::<Vec<_>>();
	html! {
		<ui::S2>
			<ui::H2 center={false}>{"Output"}</ui::H2>
			<ui::Card>
				<ui::NumberChart title="Prediction" value={props.class_name} />
			</ui::Card>
			<ui::Card>
				<ui::NumberChart
					title="Probability"
					value={ui::format_percent(props.probability)}
				/>
			</ui::Card>
			<ui::H2 center={false}>{"Explanation"}</ui::H2>
			<ui::P>
				{"This chart shows how the input values influenced the model's output."}
			</ui::P>
			<ui::Card>
				<FeatureContributionsChart
					class={None}
					title={None}
					id={"binary_classification_feature_contributions".to_owned()}
					include_x_axis_title={true}
					include_y_axis_labels={true}
					include_y_axis_title={true}
					negative_color={ui::colors::RED.to_owned()}
					positive_color={ui::colors::GREEN.to_owned()}
					series={series}
				/>
			</ui::Card>
		</ui::S2>
	}
}

#[component]
pub fn MulticlassClassificationPrediction(props: MulticlassClassificationPredictionResultProps) {
	let probability_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		title: Some("Probabilities".to_owned()),
		data: props
			.probabilities
			.iter()
			.enumerate()
			.map(|(index, (class_name, probability))| BarChartPoint {
				label: class_name.to_owned(),
				x: index.to_f64().unwrap(),
				y: Some(probability.to_f64().unwrap()),
			})
			.collect::<Vec<_>>(),
	}];
	let feature_contributions_series = props
		.feature_contributions_chart_series
		.iter()
		.map(|feature_contribution| FeatureContributionsChartSeries {
			title: feature_contribution.label.clone(),
			baseline: feature_contribution.baseline.to_f64().unwrap(),
			baseline_label: feature_contribution.baseline_label.to_owned(),
			output: feature_contribution.output.to_f64().unwrap(),
			output_label: feature_contribution.output_label.to_owned(),
			values: feature_contribution
				.values
				.iter()
				.map(|value| FeatureContributionsChartValue {
					feature: value.feature.to_owned(),
					value: value.value.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
		})
		.collect::<Vec<_>>();
	html! {
		<ui::S2>
			<ui::H2 center={false}>{"Output"}</ui::H2>
			<ui::Card>
				<ui::NumberChart
					title="Prediction"
					value={props.class_name}
				/>
			</ui::Card>
			<ui::Card>
				<ui::NumberChart
					title="Probability"
					value={ui::format_percent(props.probability)}
				/>
			</ui::Card>
			<BarChart
				class={None}
				group_gap={None}
				hide_legend={None}
				id={"probabilities".to_owned()}
				series={probability_series}
				should_draw_x_axis_labels={None}
				should_draw_y_axis_labels={None}
				title={"Predicted Probabilities".to_owned()}
				x_axis_title={None}
				y_axis_grid_line_interval={None}
				y_axis_title={None}
				y_max={None}
				y_min={None}
			/>
			<ui::H2 center={false}>{"Explanation"}</ui::H2>
			<ui::P>
				{"This chart shows how the input values influenced the model's output."}
			</ui::P>
			<ui::Card>
				<FeatureContributionsChart
					class={None}
					id={"multiclass_classification_feature_contributions".to_owned()}
					include_x_axis_title={true}
					include_y_axis_labels={true}
					include_y_axis_title={true}
					negative_color={ui::colors::RED.to_owned()}
					positive_color={ui::colors::GREEN.to_owned()}
					series={feature_contributions_series}
					title={None}
				/>
			</ui::Card>
		</ui::S2>
	}
}
