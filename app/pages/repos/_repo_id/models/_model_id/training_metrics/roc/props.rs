use crate::common::model_layout_info::ModelLayoutInfo;
use crate::{
	common::{
		error::Error,
		model::get_model,
		model_layout_info::get_model_layout_info,
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request};
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	id: String,
	roc_curve_data: Vec<ROCCurveData>,
	model_layout_info: ModelLayoutInfo,
	class: String,
	auc_roc: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ROCCurveData {
	false_positive_rate: f32,
	true_positive_rate: f32,
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
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
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
			let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
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
