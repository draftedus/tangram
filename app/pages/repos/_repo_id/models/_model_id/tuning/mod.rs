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
use sqlx::prelude::*;
use tangram_core::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Option<Inner>,
	title: String,
	model_layout_props: types::ModelLayoutProps,
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

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_model_id/tuning", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let rows = sqlx::query(
		"
			select
				id,
				title,
				created_at,
				data
			from models
			where
				models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let id: String = row.get(0);
	let title: String = row.get(1);
	let data: String = row.get(3);
	let data: Vec<u8> = base64::decode(data);
	let model = tangram_core::types::Model::from_slice(&data)?;
	let inner = match model {
		tangram_core::types::Model::Classifier(model) => {
			let classes = model.classes().to_owned();
			match model.model.into_option().unwrap() {
				tangram_core::types::ClassificationModel::UnknownVariant(_, _, _) => {
					unimplemented!()
				}
				tangram_core::types::ClassificationModel::LinearBinary(inner_model) => {
					let class_metrics = inner_model.class_metrics.into_option().unwrap();
					let metrics = build_threshold_class_metrics(class_metrics);
					Some(Inner {
						baseline_threshold: 0.5,
						metrics,
						classes,
					})
				}
				tangram_core::types::ClassificationModel::LinearMulticlass(_) => None,
				tangram_core::types::ClassificationModel::GbtBinary(inner_model) => {
					let class_metrics = inner_model.class_metrics.into_option().unwrap();
					let metrics = build_threshold_class_metrics(class_metrics);
					Some(Inner {
						baseline_threshold: 0.5,
						metrics,
						classes,
					})
				}
				tangram_core::types::ClassificationModel::GbtMulticlass(_) => None,
			}
		}
		tangram_core::types::Model::Regressor(_) => None,
		_ => return Err(Error::BadRequest.into()),
	};
	let model_layout_props = get_model_layout_props(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		inner,
		id: id.to_string(),
		title,
		model_layout_props,
	})
}

fn build_threshold_class_metrics(
	class_metrics: Vec<tangram_core::types::BinaryClassifierClassMetrics>,
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
