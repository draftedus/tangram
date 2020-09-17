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
use tangram_id::Id;

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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	tuning: Option<Inner>,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Inner {
	baseline_threshold: f32,
	metrics: Vec<Metrics>,
	class: String,
}

#[derive(serde::Serialize)]
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
	let tuning = match model {
		tangram_core::model::Model::Classifier(model) => {
			let classes = model.classes().to_owned();
			match model.model {
				tangram_core::model::ClassificationModel::LinearBinary(inner_model) => {
					let metrics = build_threshold_metrics(inner_model.metrics);
					Some(Inner {
						baseline_threshold: 0.5,
						metrics,
						class: classes[1].to_owned(),
					})
				}
				tangram_core::model::ClassificationModel::LinearMulticlass(_) => None,
				tangram_core::model::ClassificationModel::TreeBinary(inner_model) => {
					let metrics = build_threshold_metrics(inner_model.metrics);
					Some(Inner {
						baseline_threshold: 0.5,
						metrics,
						class: classes[1].to_owned(),
					})
				}
				tangram_core::model::ClassificationModel::TreeMulticlass(_) => None,
			}
		}
		tangram_core::model::Model::Regressor(_) => None,
	};
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		tuning,
		model_layout_info,
	})
}

fn build_threshold_metrics(metrics: tangram_core::model::BinaryClassifierMetrics) -> Vec<Metrics> {
	metrics
		.thresholds
		.iter()
		.map(|class_metrics| Metrics {
			threshold: class_metrics.threshold,
			precision: class_metrics.precision,
			recall: class_metrics.recall,
			accuracy: class_metrics.accuracy,
			f1_score: class_metrics.f1_score,
			false_negatives: class_metrics.false_negatives,
			false_positives: class_metrics.false_positives,
			true_negatives: class_metrics.true_negatives,
			true_positives: class_metrics.true_positives,
		})
		.collect::<Vec<Metrics>>()
}
