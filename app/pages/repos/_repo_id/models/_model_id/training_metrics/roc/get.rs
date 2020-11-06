use super::props::{Props, ROCCurveData};
use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
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
	let props = match model {
		tangram_core::model::Model::BinaryClassifier(model) => {
			let metrics = &model.test_metrics;
			let roc_curve_data = metrics
				.thresholds
				.iter()
				.map(|class_metrics| ROCCurveData {
					false_positive_rate: class_metrics.false_positive_rate,
					true_positive_rate: class_metrics.true_positive_rate,
				})
				.collect();
			let auc_roc = metrics.auc_roc;
			let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
			db.commit().await?;
			Props {
				id: model_id.to_string(),
				class: model.positive_class,
				roc_curve_data,
				auc_roc,
				model_layout_info,
			}
		}
		_ => {
			db.commit().await?;
			return Ok(bad_request());
		}
	};
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
