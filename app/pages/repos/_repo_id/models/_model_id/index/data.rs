use crate::app::{
	error::Error,
	pages::repos::new::actions::get_repo_for_model,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use tangram::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverviewViewModel {
	id: String,
	inner: Inner,
	title: String,
	repo: types::Repo,
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

pub async fn data(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;

	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	// get the necessary data from the model
	let rows = db
		.query(
			"
				select
					id,
					title,
					created_at,
					data
				from models
				where
					models.id = $1
			",
			&[&model_id],
		)
		.await?;
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let id: Id = row.get(0);
	let title: String = row.get(1);
	let data: Vec<u8> = row.get(3);
	let model = tangram::types::Model::from_slice(&data)?;
	// assemble the response
	let training_summary = training_summary(&model);
	let inner = match model {
		tangram::types::Model::Classifier(model) => {
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
		tangram::types::Model::Regressor(model) => {
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
	let response = OverviewViewModel {
		id: id.to_string(),
		title,
		inner,
		repo: get_repo_for_model(&db, id).await?,
	};
	let response = serde_json::to_vec(&response)?;

	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}

fn training_summary(model: &tangram::types::Model) -> TrainingSummary {
	let chosen_model_type_name = model_type_name(model);
	match model {
		tangram::types::Model::Regressor(model) => TrainingSummary {
			chosen_model_type_name,
			column_count: model.overall_column_stats.as_option().unwrap().len() + 1,
			model_comparison_metric_type_name: regression_model_comparison_type_name(
				model.comparison_metric.as_option().unwrap(),
			),
			row_count: model.row_count.as_option().unwrap().to_usize().unwrap(),
			test_fraction: *model.test_fraction.as_option().unwrap(),
		},
		tangram::types::Model::Classifier(model) => TrainingSummary {
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
	comparison_metric: &tangram::types::RegressionComparisonMetric,
) -> String {
	match comparison_metric {
		tangram::types::RegressionComparisonMetric::MeanAbsoluteError => {
			"Mean Absolute Error".into()
		}
		tangram::types::RegressionComparisonMetric::MeanSquaredError => "Mean Squared Error".into(),
		tangram::types::RegressionComparisonMetric::RootMeanSquaredError => {
			"Root Mean Squared Error".into()
		}
		tangram::types::RegressionComparisonMetric::R2 => "R2".into(),
		_ => unimplemented!(),
	}
}

fn classification_model_comparison_type_name(
	comparison_metric: &tangram::types::ClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram::types::ClassificationComparisonMetric::Accuracy => "Accuracy".into(),
		tangram::types::ClassificationComparisonMetric::Aucroc => {
			"Area Under the Receiver Operating Characteristic".into()
		}
		tangram::types::ClassificationComparisonMetric::F1 => "F1 Score".into(),
		_ => unimplemented!(),
	}
}

fn model_type_name(model: &tangram::types::Model) -> String {
	match model {
		tangram::types::Model::Regressor(model) => match &model.model.as_option().unwrap() {
			tangram::types::RegressionModel::Linear(_) => "Linear Regressor".into(),
			tangram::types::RegressionModel::Gbt(_) => "Gradient Boosted Tree Regressor".into(),
			_ => unimplemented!(),
		},
		tangram::types::Model::Classifier(model) => match &model.model.as_option().unwrap() {
			tangram::types::ClassificationModel::LinearBinary(_) => {
				"Linear Binary Classifier".into()
			}
			tangram::types::ClassificationModel::GbtBinary(_) => {
				"Gradient Boosted Tree Binary Classifier".into()
			}
			tangram::types::ClassificationModel::LinearMulticlass(_) => {
				"Linear Multiclass Classifier".into()
			}
			tangram::types::ClassificationModel::GbtMulticlass(_) => {
				"Gradient Boosted Tree Multiclass Classifier".into()
			}
			_ => unimplemented!(),
		},
		_ => unimplemented!(),
	}
}
