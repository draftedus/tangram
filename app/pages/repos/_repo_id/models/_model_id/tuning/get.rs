use super::props::{Inner, Metrics, Props};
use crate::{
	common::{
		error::Error,
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let model = get_model(&mut db, model_id).await?;
	let tuning = match model {
		tangram_core::model::Model::Regressor(_) => None,
		tangram_core::model::Model::BinaryClassifier(model) => {
			let metrics = model
				.test_metrics
				.thresholds
				.iter()
				.map(|metrics| Metrics {
					threshold: metrics.threshold,
					precision: metrics.precision,
					recall: metrics.recall,
					accuracy: metrics.accuracy,
					f1_score: metrics.f1_score,
					false_negatives: metrics.false_negatives,
					false_positives: metrics.false_positives,
					true_negatives: metrics.true_negatives,
					true_positives: metrics.true_positives,
				})
				.collect::<Vec<Metrics>>();
			Some(Inner {
				baseline_threshold: 0.5,
				metrics,
				class: model.positive_class,
			})
		}
		tangram_core::model::Model::MulticlassClassifier(_) => None,
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	let props = Props {
		tuning,
		model_layout_info,
	};
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/tuning", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
