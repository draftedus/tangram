use super::props::{PrecisionRecallPoint, Props};
use crate::Context;
use crate::{
	common::{
		error::Error,
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
	let props = props(context, request, model_id).await?;
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

pub async fn props(context: &Context, request: Request<Body>, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let model = get_model(&mut db, model_id).await?;
	let model = match model {
		tangram_core::model::Model::BinaryClassifier(model) => model,
		_ => return Err(Error::BadRequest.into()),
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
	db.commit().await?;
	Ok(Props {
		class: model.positive_class,
		precision_recall_curve_data,
		id: model_id.to_string(),
		model_layout_info,
	})
}
