use crate::{
	error::Error,
	helpers::repos::get_model_layout_props,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	title: String,
	classes: Vec<String>,
	non_parametric_precision_recall_curve_data: Vec<Vec<NonParametricPrecisionRecallCurveData>>,
	parametric_precision_recall_curve_data: Vec<Vec<ParametricPrecisionRecallCurveData>>,
	model_layout_props: types::ModelLayoutProps,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ParametricPrecisionRecallCurveData {
	precision: f32,
	recall: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NonParametricPrecisionRecallCurveData {
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
	let html = context
		.pinwheel
		.render(
			"/repos/_repo_id/models/_model_id/training_metrics/precision_recall",
			props,
		)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	// get the necessary data from the model
	let rows = db
		.query(
			"
				select
					id,
					title,
					created_at,
					data
				from models
				where
					models.id = $1
			",
			&[&model_id],
		)
		.await?;
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let id: Id = row.get(0);
	let title: String = row.get(1);
	let data: Vec<u8> = row.get(3);
	let model = tangram_core::types::Model::from_slice(&data)?;
	// assemble the response
	match model {
		tangram_core::types::Model::Classifier(model) => {
			let class_metrics = match model.model.as_option().unwrap() {
				tangram_core::types::ClassificationModel::LinearBinary(inner_model) => {
					inner_model.class_metrics.as_option().unwrap()
				}
				tangram_core::types::ClassificationModel::GbtBinary(inner_model) => {
					inner_model.class_metrics.as_option().unwrap()
				}
				_ => return Err(Error::BadRequest.into()),
			};
			let parametric_precision_recall_curve_data = class_metrics
				.iter()
				.map(|class_metrics| {
					class_metrics
						.thresholds
						.as_option()
						.unwrap()
						.iter()
						.map(|class_metrics| ParametricPrecisionRecallCurveData {
							precision: *class_metrics.precision.as_option().unwrap(),
							recall: *class_metrics.recall.as_option().unwrap(),
						})
						.collect()
				})
				.collect();
			let non_parametric_precision_recall_curve_data = class_metrics
				.iter()
				.map(|class_metrics| {
					class_metrics
						.thresholds
						.as_option()
						.unwrap()
						.iter()
						.map(|class_metrics| NonParametricPrecisionRecallCurveData {
							precision: *class_metrics.precision.as_option().unwrap(),
							recall: *class_metrics.recall.as_option().unwrap(),
							threshold: *class_metrics.threshold.as_option().unwrap(),
						})
						.collect()
				})
				.collect();
			let model_layout_props = get_model_layout_props(&db, model_id).await?;
			db.commit().await?;
			Ok(Props {
				id: id.to_string(),
				title,
				classes: model.classes().to_owned(),
				non_parametric_precision_recall_curve_data,
				parametric_precision_recall_curve_data,
				model_layout_props,
			})
		}
		_ => {
			db.commit().await?;
			Err(Error::BadRequest.into())
		}
	}
}
