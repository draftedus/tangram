use super::page::{render, Props, TrainedModel};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::document::PageInfo;
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_deps::{http, hyper};
use tangram_util::error::Result;
use tangram_util::id::Id;

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
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let model_comparison_metric_name = match &model {
		tangram_core::model::Model::Regressor(model) => match model.comparison_metric {
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
		},
		tangram_core::model::Model::BinaryClassifier(model) => match model.comparison_metric {
			tangram_core::model::BinaryClassificationComparisonMetric::AUCROC => "AUC".to_owned(),
		},
		tangram_core::model::Model::MulticlassClassifier(model) => match model.comparison_metric {
			tangram_core::model::MulticlassClassificationComparisonMetric::Accuracy => {
				"Accuracy".to_owned()
			}
		},
	};
	let trained_models_metrics: Vec<TrainedModel> = match &model {
		tangram_core::model::Model::Regressor(model) => model
			.grid
			.iter()
			.enumerate()
			.map(|(index, grid_item)| {
				trained_model_metrics_for_grid_item(index.to_string(), grid_item)
			})
			.collect::<Vec<_>>(),
		tangram_core::model::Model::BinaryClassifier(model) => model
			.grid
			.iter()
			.enumerate()
			.map(|(index, grid_item)| {
				trained_model_metrics_for_grid_item(index.to_string(), grid_item)
			})
			.collect::<Vec<_>>(),
		tangram_core::model::Model::MulticlassClassifier(model) => model
			.grid
			.iter()
			.enumerate()
			.map(|(index, grid_item)| {
				trained_model_metrics_for_grid_item(index.to_string(), grid_item)
			})
			.collect::<Vec<_>>(),
	};
	let best_model_metrics_index = match &model {
		tangram_core::model::Model::Regressor(model) => model.best_grid_item_index,
		tangram_core::model::Model::BinaryClassifier(model) => model.best_grid_item_index,
		tangram_core::model::Model::MulticlassClassifier(model) => model.best_grid_item_index,
	};
	let best_model_metrics = trained_models_metrics[best_model_metrics_index].clone();
	let best_model = match &model {
		tangram_core::model::Model::Regressor(model) => &model.grid[model.best_grid_item_index],
		tangram_core::model::Model::BinaryClassifier(model) => {
			&model.grid[model.best_grid_item_index]
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			&model.grid[model.best_grid_item_index]
		}
	};
	let best_model_hyperparameters = hyperparameters_for_grid_item(best_model);
	let props = Props {
		id: model_id.to_string(),
		model_comparison_metric_name,
		num_models: trained_models_metrics.len(),
		trained_models_metrics,
		best_model_metrics,
		best_model_hyperparameters,
		model_layout_info,
	};
	db.commit().await?;
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn trained_model_metrics_for_grid_item(
	identifier: String,
	grid_item: &tangram_core::model::GridItem,
) -> TrainedModel {
	match grid_item {
		tangram_core::model::GridItem::Linear(grid_item) => TrainedModel {
			identifier,
			model_comparison_metric_value: grid_item.model_comparison_metric_value,
			model_type: "Linear".into(),
			time: format!(
				"{:?}",
				std::time::Duration::from_secs_f32(grid_item.duration)
			),
		},
		tangram_core::model::GridItem::Tree(grid_item) => TrainedModel {
			identifier,
			model_comparison_metric_value: grid_item.model_comparison_metric_value,
			model_type: "Gradient Boosted Tree".into(),
			time: format!(
				"{:?}",
				std::time::Duration::from_secs_f32(grid_item.duration)
			),
		},
	}
}

fn hyperparameters_for_grid_item(
	grid_item: &tangram_core::model::GridItem,
) -> Vec<(String, String)> {
	match grid_item {
		tangram_core::model::GridItem::Linear(grid_item) => vec![
			(
				"l2_regularization".to_owned(),
				grid_item.hyperparameters.l2_regularization.to_string(),
			),
			(
				"learning_rate".to_owned(),
				grid_item.hyperparameters.learning_rate.to_string(),
			),
			(
				"max_epochs".to_owned(),
				grid_item.hyperparameters.max_epochs.to_string(),
			),
			(
				"n_examples_per_batch".to_owned(),
				grid_item.hyperparameters.n_examples_per_batch.to_string(),
			),
			(
				"early_stopping_enabled".to_owned(),
				grid_item
					.hyperparameters
					.early_stopping_options
					.is_some()
					.to_string(),
			),
		],
		tangram_core::model::GridItem::Tree(grid_item) => vec![
			(
				"early_stopping_enabled".to_owned(),
				grid_item
					.hyperparameters
					.early_stopping_options
					.is_some()
					.to_string(),
			),
			(
				"l2_regularization".to_owned(),
				grid_item.hyperparameters.l2_regularization.to_string(),
			),
			(
				"learning_rate".to_owned(),
				grid_item.hyperparameters.learning_rate.to_string(),
			),
			(
				"max_depth".to_owned(),
				grid_item
					.hyperparameters
					.max_depth
					.map(|max_depth| max_depth.to_string())
					.unwrap_or_else(|| "None".to_owned()),
			),
			(
				"max_examples_for_computing_bin_thresholds".to_owned(),
				grid_item
					.hyperparameters
					.max_examples_for_computing_bin_thresholds
					.to_string(),
			),
			(
				"max_leaf_nodes".to_owned(),
				grid_item.hyperparameters.max_leaf_nodes.to_string(),
			),
			(
				"max_rounds".to_owned(),
				grid_item.hyperparameters.max_rounds.to_string(),
			),
			(
				"max_valid_bins_for_number_features".to_owned(),
				grid_item
					.hyperparameters
					.max_valid_bins_for_number_features
					.to_string(),
			),
			(
				"min_examples_per_node".to_owned(),
				grid_item.hyperparameters.min_examples_per_node.to_string(),
			),
			(
				"min_gain_to_split".to_owned(),
				grid_item.hyperparameters.min_gain_to_split.to_string(),
			),
			(
				"min_sum_hessians_per_node".to_owned(),
				grid_item
					.hyperparameters
					.min_sum_hessians_per_node
					.to_string(),
			),
			(
				"smoothing_factor_for_discrete_bin_sorting".to_owned(),
				grid_item
					.hyperparameters
					.smoothing_factor_for_discrete_bin_sorting
					.to_string(),
			),
			(
				"supplemental_l2_regularization_for_discrete_splits".to_owned(),
				grid_item
					.hyperparameters
					.supplemental_l2_regularization_for_discrete_splits
					.to_string(),
			),
		],
	}
}
