use super::props::{Props, TrainedModelMetrics};
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
	_trained_model_identifier: &str,
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
		metrics: TrainedModelMetrics {
			identifier: "3".into(),
			metric: 81.0,
			model_type: "Linear".into(),
			time: "2 minutes".into(),
		},
		hyperparameters: vec![
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
	let html = html! {
		<ModelLayout page_info={page_info} info={props.model_layout_info} selected_item={ModelSideNavItem::TrainingGrid}>
			<ui::S1>
				<ui::H1 center={None}>{"Training Summary"}</ui::H1>
				<ui::S2>
					<ui::H2 center={None}>{"Model Metrics"}</ui::H2>
					<ModelMetricsTable trained_model={props.metrics}/>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={None}>{"Model Hyperparameters"}</ui::H2>
					<ModelHyperparametersTable hyperparameters={props.hyperparameters}/>
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
fn ModelMetricsTable(trained_model: TrainedModelMetrics) {
	let comparison_metric = "AUC ROC";
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
						{comparison_metric}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				<ui::TableRow color={None}>
					<ui::TableCell color={None} expand={None}>
						{trained_model.identifier}
					</ui::TableCell>
					<ui::TableCell color={None} expand={None}>
						{trained_model.model_type}
					</ui::TableCell>
					<ui::TableCell color={None} expand={None}>
						{trained_model.time}
					</ui::TableCell>
					<ui::TableCell color={None} expand={None}>
						{trained_model.metric.to_string()}
					</ui::TableCell>
				</ui::TableRow>
			</ui::TableBody>
		</ui::Table>
	}
}

#[component]
fn ModelHyperparametersTable(hyperparameters: Vec<(String, String)>) {
	html! {
		<ui::Table width={None}>
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
