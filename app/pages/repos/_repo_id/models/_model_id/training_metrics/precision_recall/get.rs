use super::props::{PrecisionRecallPoint, Props};
use crate::Context;
use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
};
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
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
		tangram_core::model::Model::BinaryClassifier(model) => model,
		_ => return Ok(bad_request()),
	};
	let precision_recall_curve_data = model
		.test_metrics
		.thresholds
		.iter()
		.map(|class_metrics| PrecisionRecallPoint {
			precision: class_metrics.precision,
			recall: class_metrics.recall,
			threshold: class_metrics.threshold,
		})
		.collect();
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let props = Props {
		class: model.positive_class,
		precision_recall_curve_data,
		id: model_id.to_string(),
		model_layout_info,
	};
	db.commit().await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_metrics/precision_recall",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
