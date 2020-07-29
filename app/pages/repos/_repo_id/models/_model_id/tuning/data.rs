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
struct TuningViewModel {
	id: String,
	inner: Option<Inner>,
	title: String,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Inner {
	baseline_threshold: f32,
	metrics: Vec<Vec<Metrics>>,
	classes: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
	accuracy: f32,
	f1_score: f32,
	false_negatives: u64,
	false_positives: u64,
	precision: f32,
	recall: f32,
	threshold: f32,
	true_negatives: u64,
	true_positives: u64,
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
	let inner = match model {
		tangram::types::Model::Classifier(model) => {
			let classes = model.classes().to_owned();
			match model.model.into_option().unwrap() {
				tangram::types::ClassificationModel::UnknownVariant(_, _, _) => unimplemented!(),
				tangram::types::ClassificationModel::LinearBinary(inner_model) => {
					let class_metrics = inner_model.class_metrics.into_option().unwrap();
					let metrics = build_threshold_class_metrics(class_metrics);
					Some(Inner {
						baseline_threshold: 0.5,
						metrics,
						classes,
					})
				}
				tangram::types::ClassificationModel::LinearMulticlass(_) => None,
				tangram::types::ClassificationModel::GbtBinary(inner_model) => {
					let class_metrics = inner_model.class_metrics.into_option().unwrap();
					let metrics = build_threshold_class_metrics(class_metrics);
					Some(Inner {
						baseline_threshold: 0.5,
						metrics,
						classes,
					})
				}
				tangram::types::ClassificationModel::GbtMulticlass(_) => None,
			}
		}
		tangram::types::Model::Regressor(_) => None,
		_ => return Err(Error::BadRequest.into()),
	};
	let response = TuningViewModel {
		inner,
		id: id.to_string(),
		title,
		repo: get_repo_for_model(&db, model_id).await?,
	};
	db.commit().await?;
	let response = serde_json::to_vec(&response)?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}

fn build_threshold_class_metrics(
	class_metrics: Vec<tangram::types::BinaryClassifierClassMetrics>,
) -> Vec<Vec<Metrics>> {
	class_metrics
		.iter()
		.map(|class_metrics| {
			class_metrics
				.thresholds
				.as_option()
				.unwrap()
				.iter()
				.map(|class_metrics| Metrics {
					threshold: *class_metrics.threshold.as_option().unwrap(),
					precision: *class_metrics.precision.as_option().unwrap(),
					recall: *class_metrics.recall.as_option().unwrap(),
					accuracy: *class_metrics.accuracy.as_option().unwrap(),
					f1_score: *class_metrics.f1_score.as_option().unwrap(),
					false_negatives: *class_metrics.false_negatives.as_option().unwrap(),
					false_positives: *class_metrics.false_positives.as_option().unwrap(),
					true_negatives: *class_metrics.true_negatives.as_option().unwrap(),
					true_positives: *class_metrics.true_positives.as_option().unwrap(),
				})
				.collect::<Vec<Metrics>>()
		})
		.collect()
}
