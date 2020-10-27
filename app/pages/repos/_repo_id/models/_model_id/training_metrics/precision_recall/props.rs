use crate::{
	common::{
		error::Error,
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::{get_model_layout_info, ModelLayoutInfo},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request};
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	class: String,
	precision_recall_curve_data: Vec<PrecisionRecallPoint>,
	id: String,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrecisionRecallPoint {
	precision: f32,
	recall: f32,
	threshold: f32,
}

pub async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
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
