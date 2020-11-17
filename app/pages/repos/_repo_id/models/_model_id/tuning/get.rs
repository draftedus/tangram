use super::props::{Inner, Metrics, Props};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_deps::{http, hyper};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model = get_model(&mut db, model_id).await?;
	let tuning = match model {
		tangram_core::model::Model::Regressor(_) => None,
		tangram_core::model::Model::BinaryClassifier(model) => {
			let metrics: Vec<Metrics> = model
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
				.collect();
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
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
