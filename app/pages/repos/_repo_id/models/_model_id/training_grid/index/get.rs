use super::props::{Props, TrainedModel};
use html::{component, html};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelSideNavItem},
};
use tangram_deps::{http, hyper};
use tangram_ui as ui;
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
	let _model = get_model(&mut db, model_id).await?;
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let props = Props {
		id: model_id.to_string(),
		model_comparison_metric_name: "AUC ROC".to_owned(),
		num_models: 4,
		total_training_time: "17 minutes".into(),
		trained_models_metrics: vec![
			TrainedModel {
				identifier: "1".into(),
				model_comparison_metric_value: 84.5,
				model_type: "Gradient Boosted Decision Tree".into(),
				time: "10 minutes".into(),
			},
			TrainedModel {
				identifier: "2".into(),
				model_comparison_metric_value: 87.5,
				model_type: "Linear".into(),
				time: "1 minute".into(),
			},
			TrainedModel {
				identifier: "3".into(),
				model_comparison_metric_value: 81.0,
				model_type: "Linear".into(),
				time: "2 minutes".into(),
			},
			TrainedModel {
				identifier: "4".into(),
				model_comparison_metric_value: 78.0,
				model_type: "Gradient Boosted Decision Tree".into(),
				time: "4 minutes".into(),
			},
		],
		best_model_metrics: TrainedModel {
			identifier: "5".into(),
			model_comparison_metric_value: 78.0,
			model_type: "Gradient Boosted Decision Tree".into(),
			time: "4 minutes".into(),
		},
		best_model_hyperparameters: vec![
			("l2_regularization".into(), "5".into()),
			("learning_rate".into(), "2".into()),
			("max_depth".into(), "10".into()),
			(
				"max_examples_for_computing_bin_thresholds".into(),
				"4".into(),
			),
			("max_leaf_nodes".into(), "22".into()),
		],
		model_layout_info,
	};
	db.commit().await?;
	let description = format!(
		"Tangram trained {} models. The total training time was {}.",
		props.num_models, props.total_training_time
	);
	let html = html! {
		<ModelLayout page_info={page_info} info={props.model_layout_info} selected_item={ModelSideNavItem::TrainingGrid}>
			<ui::S1>
				<ui::H1 center={None}>{"Training Summary"}</ui::H1>
				<ui::P>
					{description}
				</ui::P>
				<ui::S2>
					<ui::H2 center={None}>{"Best Model Metrics"}</ui::H2>
					<WinningModelMetricsTable best_model={props.best_model_metrics} model_comparison_metric_name={props.model_comparison_metric_name.clone()}/>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={None}>{"Best Model Hyperparameters"}</ui::H2>
					<ModelHyperparametersTable hyperparameters={props.best_model_hyperparameters} />
				</ui::S2>
				<ui::S2>
					<ui::H2 center={None}>{"All Models"}</ui::H2>
					<AllTrainedModelsMetricsTable trained_models={props.trained_models_metrics} model_comparison_metric_name={props.model_comparison_metric_name}/>
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	let body = html.render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(body))
		.unwrap();
	Ok(response)
}

#[component]
fn WinningModelMetricsTable(best_model: TrainedModel, model_comparison_metric_name: String) {
	html! {
		<ui::Table width={Some("100%".to_owned())}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Number"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Type"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Training Time"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						{model_comparison_metric_name}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableRow color={None}>
				<ui::TableCell color={None} expand={None}>
					{best_model.identifier}
				</ui::TableCell>
				<ui::TableCell color={None} expand={None}>
					{best_model.model_type}
				</ui::TableCell>
				<ui::TableCell color={None} expand={None}>
					{best_model.time}
				</ui::TableCell>
				<ui::TableCell color={None} expand={None}>
					{best_model.model_comparison_metric_value.to_string()}
				</ui::TableCell>
			</ui::TableRow>
		</ui::Table>
	}
}

#[component]
fn AllTrainedModelsMetricsTable(
	trained_models: Vec<TrainedModel>,
	model_comparison_metric_name: String,
) {
	html! {
		<ui::Table width={Some("100%".to_owned())}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Number"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Model Type"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						"Training Time"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell color={None} text_align={None} expand={None}>
						{model_comparison_metric_name}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			{trained_models.into_iter().map(|trained_model| {
				html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={None}>
							<ui::Link title={None} href={Some(format!("./trained_models/{}", trained_model.identifier))} class={None}>
								{trained_model.identifier}
							</ui::Link>
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{trained_model.model_type}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{trained_model.time}
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							{trained_model.model_comparison_metric_value.to_string()}
						</ui::TableCell>
					</ui::TableRow>
				}
			}).collect::<Vec<_>>()}
		</ui::Table>
	}
}

#[component]
fn ModelHyperparametersTable(hyperparameters: Vec<(String, String)>) {
	html! {
		<ui::Table width={Some("100%".to_owned())}>
		{hyperparameters.into_iter().map(|(hyperparam_name, hyperparam_value)| {
			html! {
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} text_align={None} expand={Some(false)}>
						{hyperparam_name}
					</ui::TableHeaderCell>
					<ui::TableCell color={None} expand={Some(true)}>
						{hyperparam_value}
					</ui::TableCell>
				</ui::TableRow>
			}
		}).collect::<Vec<_>>()}
		</ui::Table>
	}
}
