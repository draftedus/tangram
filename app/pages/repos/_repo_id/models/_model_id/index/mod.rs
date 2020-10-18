use crate::{
	common::{
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value")]
enum Inner {
	#[serde(rename = "regressor")]
	Regressor(RegressorInner),
	#[serde(rename = "binary_classifier")]
	BinaryClassifier(BinaryClassifierInner),
	#[serde(rename = "multiclass_classifier")]
	MulticlassClassifier(MulticlassClassifierInner),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorInner {
	id: String,
	metrics: RegressorInnerMetrics,
	training_summary: TrainingSummary,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorInnerMetrics {
	baseline_mse: f32,
	baseline_rmse: f32,
	mse: f32,
	rmse: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BinaryClassifierInner {
	id: String,
	metrics: BinaryClassifierInnerMetrics,
	training_summary: TrainingSummary,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BinaryClassifierInnerMetrics {
	auc_roc: f32,
	accuracy: f32,
	precision: f32,
	recall: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifierInner {
	id: String,
	metrics: MulticlassClassifierInnerMetrics,
	training_summary: TrainingSummary,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifierInnerMetrics {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<MulticlassClassifierInnerClassMetrics>,
	classes: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifierInnerClassMetrics {
	precision: f32,
	recall: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingSummary {
	chosen_model_type_name: String,
	column_count: usize,
	model_comparison_metric_type_name: String,
	train_row_count: usize,
	test_row_count: usize,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let model = get_model(&mut db, model_id).await?;
	let training_summary = training_summary(&model);
	let inner = match &model {
		tangram_core::model::Model::Regressor(model) => Inner::Regressor(RegressorInner {
			id: model_id.to_string(),
			metrics: RegressorInnerMetrics {
				rmse: model.test_metrics.rmse,
				baseline_rmse: model.test_metrics.baseline_rmse,
				mse: model.test_metrics.mse,
				baseline_mse: model.test_metrics.baseline_mse,
			},
			training_summary,
		}),
		tangram_core::model::Model::BinaryClassifier(model) => {
			let default_threshold_test_metrics = model
				.test_metrics
				.thresholds
				.get(model.test_metrics.thresholds.len() / 2)
				.unwrap();
			Inner::BinaryClassifier(BinaryClassifierInner {
				id: model_id.to_string(),
				metrics: BinaryClassifierInnerMetrics {
					auc_roc: model.test_metrics.auc_roc,
					accuracy: default_threshold_test_metrics.accuracy,
					precision: default_threshold_test_metrics.precision,
					recall: default_threshold_test_metrics.recall,
				},
				training_summary,
			})
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			let class_metrics = model
				.test_metrics
				.class_metrics
				.iter()
				.map(|class_metrics| MulticlassClassifierInnerClassMetrics {
					precision: class_metrics.precision,
					recall: class_metrics.recall,
				})
				.collect::<Vec<MulticlassClassifierInnerClassMetrics>>();
			Inner::MulticlassClassifier(MulticlassClassifierInner {
				id: model_id.to_string(),
				metrics: MulticlassClassifierInnerMetrics {
					accuracy: model.test_metrics.accuracy,
					baseline_accuracy: model.test_metrics.baseline_accuracy,
					class_metrics,
					classes: model.classes().to_owned(),
				},
				training_summary,
			})
		}
	};
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}

fn training_summary(model: &tangram_core::model::Model) -> TrainingSummary {
	let chosen_model_type_name = model_type_name(model);
	match model {
		tangram_core::model::Model::Regressor(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.len() + 1,
			model_comparison_metric_type_name: regression_model_comparison_type_name(
				&model.comparison_metric,
			),
			train_row_count: model.train_row_count.to_usize().unwrap(),
			test_row_count: model.test_row_count.to_usize().unwrap(),
		},
		tangram_core::model::Model::BinaryClassifier(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.len() + 1,
			model_comparison_metric_type_name: binary_classification_model_comparison_type_name(
				&model.comparison_metric,
			),
			train_row_count: model.train_row_count.to_usize().unwrap(),
			test_row_count: model.test_row_count.to_usize().unwrap(),
		},
		tangram_core::model::Model::MulticlassClassifier(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.len() + 1,
			model_comparison_metric_type_name: multiclass_classification_model_comparison_type_name(
				&model.comparison_metric,
			),
			train_row_count: model.train_row_count.to_usize().unwrap(),
			test_row_count: model.test_row_count.to_usize().unwrap(),
		},
	}
}

fn regression_model_comparison_type_name(
	comparison_metric: &tangram_core::model::RegressionComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::model::RegressionComparisonMetric::MeanAbsoluteError => {
			"Mean Absolute Error".into()
		}
		tangram_core::model::RegressionComparisonMetric::MeanSquaredError => {
			"Mean Squared Error".into()
		}
		tangram_core::model::RegressionComparisonMetric::RootMeanSquaredError => {
			"Root Mean Squared Error".into()
		}
		tangram_core::model::RegressionComparisonMetric::R2 => "R2".into(),
	}
}

fn binary_classification_model_comparison_type_name(
	comparison_metric: &tangram_core::model::BinaryClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::model::BinaryClassificationComparisonMetric::AUCROC => {
			"Area Under the Receiver Operating Characteristic Curve".into()
		}
	}
}

fn multiclass_classification_model_comparison_type_name(
	comparison_metric: &tangram_core::model::MulticlassClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::model::MulticlassClassificationComparisonMetric::Accuracy => {
			"Accuracy".into()
		}
	}
}

fn model_type_name(model: &tangram_core::model::Model) -> String {
	match model {
		tangram_core::model::Model::Regressor(model) => match &model.model {
			tangram_core::model::RegressionModel::Linear(_) => "Linear Regressor".into(),
			tangram_core::model::RegressionModel::Tree(_) => {
				"Gradient Boosted Tree Regressor".into()
			}
		},
		tangram_core::model::Model::BinaryClassifier(model) => match &model.model {
			tangram_core::model::BinaryClassificationModel::Linear(_) => {
				"Linear Binary Classifier".into()
			}
			tangram_core::model::BinaryClassificationModel::Tree(_) => {
				"Gradient Boosted Tree Binary Classifier".into()
			}
		},
		tangram_core::model::Model::MulticlassClassifier(model) => match &model.model {
			tangram_core::model::MulticlassClassificationModel::Linear(_) => {
				"Linear Multiclass Classifier".into()
			}
			tangram_core::model::MulticlassClassificationModel::Tree(_) => {
				"Gradient Boosted Tree Multiclass Classifier".into()
			}
		},
	}
}
