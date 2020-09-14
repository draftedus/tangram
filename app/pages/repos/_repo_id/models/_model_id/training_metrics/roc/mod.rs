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
	auc_roc: f32,
	roc_curve_data: Vec<Vec<ROCCurveData>>,
	classes: Vec<String>,
	model_layout_info: ModelLayoutInfo,
	class: String,
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
			let (class_metrics, auc_roc) = match &model.model {
				tangram_core::model::ClassificationModel::LinearBinary(inner_model) => {
					(&inner_model.class_metrics, inner_model.auc_roc)
				}
				tangram_core::model::ClassificationModel::TreeBinary(inner_model) => {
					(&inner_model.class_metrics, inner_model.auc_roc)
				}
				_ => return Err(Error::BadRequest.into()),
			};
			let roc_curve_data = class_metrics
				.iter()
				.map(|class_metrics| {
					class_metrics
						.thresholds
						.iter()
						.map(|class_metrics| ROCCurveData {
							false_positive_rate: class_metrics.false_positive_rate,
							true_positive_rate: class_metrics.true_positive_rate,
						})
						.collect()
				})
				.collect();

			let model_layout_info = get_model_layout_info(&mut db, model_id).await?;

			db.commit().await?;

			let classes = model.classes().to_owned();
			let class_index = if let Some(class) = &class {
				classes.iter().position(|c| c == class).unwrap()
			} else {
				1
			};
			let class = class.unwrap_or_else(|| classes[class_index].to_owned());

			Ok(Props {
				id: model_id.to_string(),
				classes,
				class,
				auc_roc,
				roc_curve_data,
				model_layout_info,
			})
		}
		_ => {
			db.commit().await?;
			Err(Error::BadRequest.into())
		}
	}
}
