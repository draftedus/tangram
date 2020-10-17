use crate::{
	common::{
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	class: String,
	precision_recall_curve_data: Vec<PrecisionRecallPoint>,
	id: String,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PrecisionRecallPoint {
	precision: f32,
	recall: f32,
	threshold: f32,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
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

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
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
	let model = match model {
		tangram_core::model::Model::MulticlassClassifier(model) => model,
		_ => return Err(Error::BadRequest.into()),
	};
	let metrics = match &model.model {
		// tangram_core::model::MulticlassClassificationModel::LinearBinary(inner_model) => {
		// 	&inner_model.metrics
		// }
		// tangram_core::model::MulticlassClassificationModel::TreeBinary(inner_model) => {
		// 	&inner_model.metrics
		// }
		_ => return Err(Error::BadRequest.into()),
	};
	todo!()
	// let classes = model.classes().to_owned();
	// let class = classes[1].to_owned();
	// let precision_recall_curve_data = metrics
	// 	.thresholds
	// 	.iter()
	// 	.map(|class_metrics| PrecisionRecallPoint {
	// 		precision: class_metrics.precision,
	// 		recall: class_metrics.recall,
	// 		threshold: class_metrics.threshold,
	// 	})
	// 	.collect();
	// let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	// db.commit().await?;
	// Ok(Props {
	// 	class,
	// 	precision_recall_curve_data,
	// 	id: model_id.to_string(),
	// 	model_layout_info,
	// })
}
