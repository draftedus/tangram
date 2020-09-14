use crate::{
	common::{
		model::{get_model, Model},
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::cast::ToPrimitive;
use tangram_core::util::id::Id;

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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	Regressor(Regressor),
	Classifier(Classifier),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Regressor {
	id: String,
	metrics: RegressorMetrics,
	training_summary: TrainingSummary,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorMetrics {
	baseline_mse: f32,
	baseline_rmse: f32,
	mse: f32,
	rmse: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Classifier {
	id: String,
	metrics: ClassifierMetrics,
	training_summary: TrainingSummary,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingSummary {
	chosen_model_type_name: String,
	column_count: usize,
	model_comparison_metric_type_name: String,
	row_count: usize,
	test_fraction: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassifierMetrics {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetrics {
	precision: f32,
	recall: f32,
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
	let Model { data, id } = get_model(&mut db, model_id).await?;
	let model = tangram_core::model::Model::from_slice(&data)?;
	// assemble the response
	let training_summary = training_summary(&model);
	let inner = match &model {
		tangram_core::model::Model::Classifier(model) => {
			let test_metrics = &model.test_metrics;
			let class_metrics = &test_metrics.class_metrics;
			let class_metrics = class_metrics
				.iter()
				.map(|class_metrics| ClassMetrics {
					precision: class_metrics.precision,
					recall: class_metrics.recall,
				})
				.collect::<Vec<ClassMetrics>>();
			Inner::Classifier(Classifier {
				id: id.to_string(),
				metrics: ClassifierMetrics {
					accuracy: test_metrics.accuracy,
					baseline_accuracy: test_metrics.baseline_accuracy,
					class_metrics,
					classes: model.classes().to_owned(),
				},
				training_summary,
			})
		}
		tangram_core::model::Model::Regressor(model) => {
			let test_metrics = &model.test_metrics;
			Inner::Regressor(Regressor {
				id: id.to_string(),
				metrics: RegressorMetrics {
					rmse: test_metrics.rmse,
					baseline_rmse: test_metrics.baseline_rmse,
					mse: test_metrics.mse,
					baseline_mse: test_metrics.baseline_mse,
				},
				training_summary,
			})
		}
	};

	let model_layout_info = get_model_layout_info(&mut db, id).await?;
	db.commit().await?;

	Ok(Props {
		id: id.to_string(),
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
			row_count: model.row_count.to_usize().unwrap(),
			test_fraction: model.test_fraction,
		},
		tangram_core::model::Model::Classifier(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.len() + 1,
			model_comparison_metric_type_name: classification_model_comparison_type_name(
				&model.comparison_metric,
			),
			row_count: model.row_count.to_usize().unwrap(),
			test_fraction: model.test_fraction,
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

fn classification_model_comparison_type_name(
	comparison_metric: &tangram_core::model::ClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::model::ClassificationComparisonMetric::Accuracy => "Accuracy".into(),
		tangram_core::model::ClassificationComparisonMetric::Aucroc => {
			"Area Under the Receiver Operating Characteristic".into()
		}
		tangram_core::model::ClassificationComparisonMetric::F1 => "F1 Score".into(),
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
		tangram_core::model::Model::Classifier(model) => match &model.model {
			tangram_core::model::ClassificationModel::LinearBinary(_) => {
				"Linear Binary Classifier".into()
			}
			tangram_core::model::ClassificationModel::TreeBinary(_) => {
				"Gradient Boosted Tree Binary Classifier".into()
			}
			tangram_core::model::ClassificationModel::LinearMulticlass(_) => {
				"Linear Multiclass Classifier".into()
			}
			tangram_core::model::ClassificationModel::TreeMulticlass(_) => {
				"Gradient Boosted Tree Multiclass Classifier".into()
			}
		},
	}
}
