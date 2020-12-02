use super::page::{
	render, BinaryClassifierInnerMetrics, BinaryClassifierProps, Inner,
	MulticlassClassifierInnerClassMetrics, MulticlassClassifierInnerMetrics,
	MulticlassClassifierProps, Props, RegressorInnerMetrics, RegressorProps, TrainingSummary,
};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{document::PageInfo, model_layout::get_model_layout_info};
use tangram_deps::{http, hyper, num_traits::ToPrimitive};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model = get_model(&mut db, model_id).await?;
	let training_summary = training_summary(&model);
	let inner = match &model {
		tangram_core::model::Model::Regressor(model) => Inner::Regressor(RegressorProps {
			id: model_id.to_string(),
			metrics: RegressorInnerMetrics {
				rmse: model.test_metrics.rmse,
				baseline_rmse: model.baseline_metrics.rmse,
				mse: model.test_metrics.mse,
				baseline_mse: model.baseline_metrics.mse,
			},
			losses_chart_series: match &model.model {
				tangram_core::model::RegressionModel::Linear(model) => model.losses.clone(),
				tangram_core::model::RegressionModel::Tree(model) => model.losses.clone(),
			},
			training_summary,
		}),
		tangram_core::model::Model::BinaryClassifier(model) => {
			let default_threshold_test_metrics = model
				.test_metrics
				.thresholds
				.get(model.test_metrics.thresholds.len() / 2)
				.unwrap();
			let default_threshold_baseline_metrics = model
				.baseline_metrics
				.thresholds
				.get(model.baseline_metrics.thresholds.len() / 2)
				.unwrap();
			Inner::BinaryClassifier(BinaryClassifierProps {
				id: model_id.to_string(),
				metrics: BinaryClassifierInnerMetrics {
					baseline_accuracy: default_threshold_baseline_metrics.accuracy,
					auc_roc: model.test_metrics.auc_roc,
					accuracy: default_threshold_test_metrics.accuracy,
					precision: default_threshold_test_metrics.precision,
					recall: default_threshold_test_metrics.recall,
				},
				losses_chart_series: match &model.model {
					tangram_core::model::BinaryClassificationModel::Linear(model) => {
						model.losses.clone()
					}
					tangram_core::model::BinaryClassificationModel::Tree(model) => {
						model.losses.clone()
					}
				},
				training_summary,
			})
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			let class_metrics: Vec<MulticlassClassifierInnerClassMetrics> = model
				.test_metrics
				.class_metrics
				.iter()
				.map(|class_metrics| MulticlassClassifierInnerClassMetrics {
					precision: class_metrics.precision,
					recall: class_metrics.recall,
				})
				.collect();
			Inner::MulticlassClassifier(MulticlassClassifierProps {
				id: model_id.to_string(),
				metrics: MulticlassClassifierInnerMetrics {
					accuracy: model.test_metrics.accuracy,
					baseline_accuracy: model.baseline_metrics.accuracy,
					class_metrics,
					classes: model.classes.clone(),
				},
				losses_chart_series: match &model.model {
					tangram_core::model::MulticlassClassificationModel::Linear(model) => {
						model.losses.clone()
					}
					tangram_core::model::MulticlassClassificationModel::Tree(model) => {
						model.losses.clone()
					}
				},
				training_summary,
			})
		}
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	let props = Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	};
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
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
			"Mean Absolute Error".to_owned()
		}
		tangram_core::model::RegressionComparisonMetric::MeanSquaredError => {
			"Mean Squared Error".to_owned()
		}
		tangram_core::model::RegressionComparisonMetric::RootMeanSquaredError => {
			"Root Mean Squared Error".to_owned()
		}
		tangram_core::model::RegressionComparisonMetric::R2 => "R2".to_owned(),
	}
}

fn binary_classification_model_comparison_type_name(
	comparison_metric: &tangram_core::model::BinaryClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::model::BinaryClassificationComparisonMetric::AUCROC => {
			"Area Under the Receiver Operating Characteristic Curve".to_owned()
		}
	}
}

fn multiclass_classification_model_comparison_type_name(
	comparison_metric: &tangram_core::model::MulticlassClassificationComparisonMetric,
) -> String {
	match comparison_metric {
		tangram_core::model::MulticlassClassificationComparisonMetric::Accuracy => {
			"Accuracy".to_owned()
		}
	}
}

fn model_type_name(model: &tangram_core::model::Model) -> String {
	match model {
		tangram_core::model::Model::Regressor(model) => match &model.model {
			tangram_core::model::RegressionModel::Linear(_) => "Linear Regressor".to_owned(),
			tangram_core::model::RegressionModel::Tree(_) => {
				"Gradient Boosted Tree Regressor".to_owned()
			}
		},
		tangram_core::model::Model::BinaryClassifier(model) => match &model.model {
			tangram_core::model::BinaryClassificationModel::Linear(_) => {
				"Linear Binary Classifier".to_owned()
			}
			tangram_core::model::BinaryClassificationModel::Tree(_) => {
				"Gradient Boosted Tree Binary Classifier".to_owned()
			}
		},
		tangram_core::model::Model::MulticlassClassifier(model) => match &model.model {
			tangram_core::model::MulticlassClassificationModel::Linear(_) => {
				"Linear Multiclass Classifier".to_owned()
			}
			tangram_core::model::MulticlassClassificationModel::Tree(_) => {
				"Gradient Boosted Tree Multiclass Classifier".to_owned()
			}
		},
	}
}
