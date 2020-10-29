use super::props::*;
use crate::{
	common::{
		error::Error,
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_metrics/roc",
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
	match model {
		tangram_core::model::Model::BinaryClassifier(model) => {
			let metrics = &model.test_metrics;
			let roc_curve_data = metrics
				.thresholds
				.iter()
				.map(|class_metrics| ROCCurveData {
					false_positive_rate: class_metrics.false_positive_rate,
					true_positive_rate: class_metrics.true_positive_rate,
				})
				.collect();
			let auc_roc = metrics.auc_roc;
			let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
			db.commit().await?;
			Ok(Props {
				id: model_id.to_string(),
				class: model.positive_class,
				roc_curve_data,
				auc_roc,
				model_layout_info,
			})
		}
		_ => {
			db.commit().await?;
			Err(Error::BadRequest.into())
		}
	}
}
