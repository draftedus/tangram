use crate::{
	common::{
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let class = search_params.map(|s| s.get("class").unwrap().to_owned());
	let props = props(request, context, model_id, class).await?;
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	roc_curve_data: Vec<ROCCurveData>,
	classes: Vec<String>,
	model_layout_info: ModelLayoutInfo,
	class: String,
	auc_roc: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ROCCurveData {
	false_positive_rate: f32,
	true_positive_rate: f32,
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	class: Option<String>,
) -> Result<Props> {
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
		tangram_core::model::Model::Classifier(model) => {
			let metrics = match &model.model {
				tangram_core::model::ClassificationModel::LinearBinary(inner_model) => {
					&inner_model.metrics
				}
				tangram_core::model::ClassificationModel::TreeBinary(inner_model) => {
					&inner_model.metrics
				}
				_ => return Err(Error::BadRequest.into()),
			};
			let classes = model.classes().to_owned();
			let class = class.unwrap_or_else(|| classes[1].to_owned());
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
				classes,
				class,
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
