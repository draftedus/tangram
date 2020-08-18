use crate::{
	error::Error,
	helpers::{
		model::{get_model, Model},
		repos::get_model_layout_info,
	},
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/tuning", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	tuning: Option<Inner>,
	model_layout_info: types::ModelLayoutInfo,
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
	let Model { data, .. } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(&data)?;
	let tuning = match model {
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
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		tuning,
		model_layout_info,
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
