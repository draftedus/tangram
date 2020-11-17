use super::props::Props;
use std::collections::BTreeMap;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_deps::{http, hyper};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
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
	let model = match model {
		tangram_core::model::Model::MulticlassClassifier(model) => model,
		_ => return Ok(bad_request()),
	};
	let class = search_params.map(|s| s.get("class").unwrap().clone());
	let classes = model.classes.clone();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		0
	};
	let class = class.unwrap_or_else(|| classes.get(class_index).unwrap().clone());
	let class_metrics = &model.test_metrics.class_metrics[class_index];
	let precision = class_metrics.precision;
	let recall = class_metrics.recall;
	let f1_score = class_metrics.f1_score;
	let true_negatives = class_metrics.true_negatives;
	let true_positives = class_metrics.true_positives;
	let false_negatives = class_metrics.false_negatives;
	let false_positives = class_metrics.false_positives;
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	let props = Props {
		id: model_id.to_string(),
		model_layout_info,
		class,
		classes,
		f1_score,
		false_negatives,
		false_positives,
		precision,
		recall,
		true_negatives,
		true_positives,
	};
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_metrics/class_metrics",
		props,
	)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
