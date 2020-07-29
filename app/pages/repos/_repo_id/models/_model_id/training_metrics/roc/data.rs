use crate::app::{
	error::Error,
	pages::repos::new::actions::get_repo_for_model,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingMetricsROCViewModel {
	id: String,
	title: String,
	auc_roc: f32,
	roc_curve_data: Vec<Vec<ROCCurveData>>,
	classes: Vec<String>,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ROCCurveData {
	false_positive_rate: f32,
	true_positive_rate: f32,
}

pub async fn data(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
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
	let model = tangram::types::Model::from_slice(&data)?;
	// assemble the response
	let response = match model {
		tangram::types::Model::Classifier(model) => {
			let (class_metrics, auc_roc) = match model.model.as_option().unwrap() {
				tangram::types::ClassificationModel::LinearBinary(inner_model) => (
					inner_model.class_metrics.as_option().unwrap(),
					inner_model.auc_roc.as_option().unwrap(),
				),
				tangram::types::ClassificationModel::GbtBinary(inner_model) => (
					inner_model.class_metrics.as_option().unwrap(),
					inner_model.auc_roc.as_option().unwrap(),
				),
				_ => return Err(Error::BadRequest.into()),
			};
			let roc_curve_data = class_metrics
				.iter()
				.map(|class_metrics| {
					class_metrics
						.thresholds
						.as_option()
						.unwrap()
						.iter()
						.map(|class_metrics| ROCCurveData {
							false_positive_rate: *class_metrics
								.false_positive_rate
								.as_option()
								.unwrap(),
							true_positive_rate: *class_metrics
								.true_positive_rate
								.as_option()
								.unwrap(),
						})
						.collect()
				})
				.collect();
			TrainingMetricsROCViewModel {
				id: id.to_string(),
				title,
				classes: model.classes().to_owned(),
				auc_roc: *auc_roc,
				roc_curve_data,
				repo: get_repo_for_model(&db, model_id).await?,
			}
		}
		_ => return Err(Error::BadRequest.into()),
	};

	let response = serde_json::to_vec(&response)?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}
