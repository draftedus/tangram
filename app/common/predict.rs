use std::convert::TryInto;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionResult {
	pub input_table: InputTable,
	pub prediction: Prediction,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTable {
	pub rows: Vec<InputTableRow>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTableRow {
	pub column_name: String,
	pub column_type: ColumnType,
	pub value: serde_json::Value,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Prediction {
	Regression(RegressionPrediction),
	BinaryClassification(BinaryClassificationPrediction),
	MulticlassClassification(MulticlassClassificationProductionPrediction),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPrediction {
	value: f32,
	feature_contributions_chart_data: FeatureContributionsChartData,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationPrediction {
	class_name: String,
	probability: f32,
	feature_contributions_chart_data: FeatureContributionsChartData,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassificationProductionPrediction {
	class_name: String,
	probability: f32,
	probabilities: Vec<(String, f32)>,
	feature_contributions_chart_data: FeatureContributionsChartData,
}

pub type FeatureContributionsChartData = Vec<FeatureContributionsChartSeries>;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeatureContributionsChartSeries {
	baseline: f32,
	baseline_label: String,
	label: String,
	output: f32,
	output_label: String,
	values: Vec<FeatureContributionsChartValue>,
}

#[derive(serde::Serialize, Debug)]
pub struct FeatureContributionsChartValue {
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
			let feature_contributions_chart_data = vec![FeatureContributionsChartSeries {
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
			let prediction = RegressionPrediction {
				feature_contributions_chart_data,
				value: output.value,
			};
			Prediction::Regression(prediction)
		}
		tangram_core::predict::PredictOutput::BinaryClassification(mut output) => {
			let output = output.remove(0);
			let feature_contributions = output.feature_contributions.unwrap();
			let feature_contributions_chart_data = vec![FeatureContributionsChartSeries {
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
			let prediction = BinaryClassificationPrediction {
				class_name: output.class_name,
				probability: output.probability,
				feature_contributions_chart_data,
			};
			Prediction::BinaryClassification(prediction)
		}
		tangram_core::predict::PredictOutput::MulticlassClassification(mut output) => {
			let output = output.remove(0);
			let feature_contributions_chart_data: Vec<FeatureContributionsChartSeries> = output
				.feature_contributions
				.unwrap()
				.into_iter()
				.map(
					|(class, feature_contributions)| FeatureContributionsChartSeries {
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
			let prediction = MulticlassClassificationProductionPrediction {
				class_name: output.class_name,
				probability: output.probability,
				probabilities: output.probabilities.into_iter().collect::<Vec<_>>(),
				feature_contributions_chart_data,
			};
			Prediction::MulticlassClassification(prediction)
		}
	};
	predict_output
}

fn compute_feature_contributions_chart_value(
	feature_contribution: tangram_core::predict::FeatureContribution,
) -> FeatureContributionsChartValue {
	match feature_contribution {
		tangram_core::predict::FeatureContribution::Identity {
			column_name,
			feature_contribution_value,
		} => FeatureContributionsChartValue {
			feature: column_name,
			value: feature_contribution_value,
		},
		tangram_core::predict::FeatureContribution::Normalized {
			column_name,
			feature_contribution_value,
		} => FeatureContributionsChartValue {
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
			FeatureContributionsChartValue {
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
			FeatureContributionsChartValue {
				feature,
				value: feature_contribution_value,
			}
		}
	}
}
