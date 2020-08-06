use crate::{
	error::Error,
	helpers::{
		model::{get_model, Model},
		repos::get_model_layout_props,
	},
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_model_id/", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	title: String,
	model_layout_props: types::ModelLayoutProps,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	Regressor(Regressor),
	Classifier(Classifier),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Regressor {
	id: String,
	metrics: RegressorMetrics,
	training_summary: TrainingSummary,
	title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorMetrics {
	baseline_mse: f32,
	baseline_rmse: f32,
	mse: f32,
	rmse: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Classifier {
	id: String,
	metrics: ClassifierMetrics,
	training_summary: TrainingSummary,
	title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingSummary {
	chosen_model_type_name: String,
	column_count: usize,
	model_comparison_metric_type_name: String,
	row_count: usize,
	test_fraction: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassifierMetrics {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
}

#[derive(Serialize)]
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
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;

	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let Model { title, data, id } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(&data)?;
	// assemble the response
	let training_summary = training_summary(&model);
	let inner = match model {
		tangram_core::types::Model::Classifier(model) => {
			let test_metrics = model.test_metrics.as_option().unwrap();
			let class_metrics = test_metrics.class_metrics.as_option().unwrap();
			let class_metrics = class_metrics
				.iter()
				.map(|class_metrics| ClassMetrics {
					precision: *class_metrics.precision.as_option().unwrap(),
					recall: *class_metrics.recall.as_option().unwrap(),
				})
				.collect::<Vec<ClassMetrics>>();
			Inner::Classifier(Classifier {
				id: id.to_string(),
				metrics: ClassifierMetrics {
					accuracy: *test_metrics.accuracy.as_option().unwrap(),
					baseline_accuracy: *test_metrics.baseline_accuracy.as_option().unwrap(),
					class_metrics,
					classes: model.classes().to_owned(),
				},
				title: title.to_owned(),
				training_summary,
			})
		}
		tangram_core::types::Model::Regressor(model) => {
			let test_metrics = model.test_metrics.as_option().unwrap();
			Inner::Regressor(Regressor {
				id: id.to_string(),
				metrics: RegressorMetrics {
					rmse: *test_metrics.rmse.as_option().unwrap(),
					baseline_rmse: *test_metrics.baseline_rmse.as_option().unwrap(),
					mse: *test_metrics.mse.as_option().unwrap(),
					baseline_mse: *test_metrics.baseline_mse.as_option().unwrap(),
				},
				training_summary,
				title: title.to_owned(),
			})
		}
		_ => return Err(Error::BadRequest.into()),
	};

	let model_layout_props = get_model_layout_props(&mut db, id).await?;
	db.commit().await?;

	Ok(Props {
		id: id.to_string(),
		title,
		inner,
		model_layout_props,
	})
}

fn training_summary(model: &tangram_core::types::Model) -> TrainingSummary {
	let chosen_model_type_name = model_type_name(model);
	match model {
		tangram_core::types::Model::Regressor(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.as_option().unwrap().len() + 1,
			model_comparison_metric_type_name: regression_model_comparison_type_name(
				model.comparison_metric.as_option().unwrap(),
			),
			row_count: model.row_count.as_option().unwrap().to_usize().unwrap(),
			test_fraction: *model.test_fraction.as_option().unwrap(),
		},
		tangram_core::types::Model::Classifier(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.as_option().unwrap().len() + 1,
			model_comparison_metric_type_name: classification_model_comparison_type_name(
				model.comparison_metric.as_option().unwrap(),
			),
			row_count: model.row_count.as_option().unwrap().to_usize().unwrap(),
			test_fraction: *model.test_fraction.as_option().unwrap(),
		},
		_ => unimplemented!(),
	}
}

fn regression_model_comparison_type_name(
	comparison_metric: &tangram_core::types::RegressionComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::types::RegressionComparisonMetric::MeanAbsoluteError => {
			"Mean Absolute Error".into()
		}
		tangram_core::types::RegressionComparisonMetric::MeanSquaredError => {
			"Mean Squared Error".into()
		}
		tangram_core::types::RegressionComparisonMetric::RootMeanSquaredError => {
			"Root Mean Squared Error".into()
		}
		tangram_core::types::RegressionComparisonMetric::R2 => "R2".into(),
		_ => unimplemented!(),
	}
}

fn classification_model_comparison_type_name(
	comparison_metric: &tangram_core::types::ClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::types::ClassificationComparisonMetric::Accuracy => "Accuracy".into(),
		tangram_core::types::ClassificationComparisonMetric::Aucroc => {
			"Area Under the Receiver Operating Characteristic".into()
		}
		tangram_core::types::ClassificationComparisonMetric::F1 => "F1 Score".into(),
		_ => unimplemented!(),
	}
}

fn model_type_name(model: &tangram_core::types::Model) -> String {
	match model {
		tangram_core::types::Model::Regressor(model) => match &model.model.as_option().unwrap() {
			tangram_core::types::RegressionModel::Linear(_) => "Linear Regressor".into(),
			tangram_core::types::RegressionModel::Gbt(_) => {
				"Gradient Boosted Tree Regressor".into()
			}
			_ => unimplemented!(),
		},
		tangram_core::types::Model::Classifier(model) => match &model.model.as_option().unwrap() {
			tangram_core::types::ClassificationModel::LinearBinary(_) => {
				"Linear Binary Classifier".into()
			}
			tangram_core::types::ClassificationModel::GbtBinary(_) => {
				"Gradient Boosted Tree Binary Classifier".into()
			}
			tangram_core::types::ClassificationModel::LinearMulticlass(_) => {
				"Linear Multiclass Classifier".into()
			}
			tangram_core::types::ClassificationModel::GbtMulticlass(_) => {
				"Gradient Boosted Tree Multiclass Classifier".into()
			}
			_ => unimplemented!(),
		},
		_ => unimplemented!(),
	}
}
