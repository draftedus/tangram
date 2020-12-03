use super::page::{render, Props, ROCCurveData};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{document::PageInfo, model_layout::get_model_layout_info};
use tangram_deps::pinwheel::{self, client};
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
	let page_info = PageInfo {
		client_wasm_js_src: Some(client!()),
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
