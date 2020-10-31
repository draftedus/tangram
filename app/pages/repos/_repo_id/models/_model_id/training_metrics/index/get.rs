use super::props::{
	BinaryClassifierProps, ClassMetrics, Inner, MulticlassClassifierProps, Props, RegressorProps,
};
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
	let props = props(context, request, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/training_metrics/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

pub async fn props(context: &Context, request: Request<Body>, model_id: &str) -> Result<Props> {
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
	let inner = match model {
		tangram_core::model::Model::Regressor(model) => {
			Inner::Regressor(build_inner_regressor(model))
		}
		tangram_core::model::Model::BinaryClassifier(model) => {
			Inner::BinaryClassifier(build_inner_binary_classifier(model))
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			Inner::MulticlassClassifier(build_inner_multiclass_classifier(model))
		}
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}

fn build_inner_regressor(model: tangram_core::model::Regressor) -> RegressorProps {
	RegressorProps {
		id: model.id,
		rmse: model.test_metrics.rmse,
		baseline_rmse: model.baseline_metrics.rmse,
		mse: model.test_metrics.mse,
		baseline_mse: model.baseline_metrics.mse,
	}
}

fn build_inner_binary_classifier(
	model: tangram_core::model::BinaryClassifier,
) -> BinaryClassifierProps {
	let default_threshold_test_metrics = model
		.test_metrics
		.thresholds
		.get(model.test_metrics.thresholds.len() / 2)
		.unwrap();
	let baseline_accuracy = model
		.baseline_metrics
		.thresholds
		.get(model.baseline_metrics.thresholds.len() / 2)
		.unwrap()
		.accuracy;
	BinaryClassifierProps {
		accuracy: default_threshold_test_metrics.accuracy,
		baseline_accuracy,
		auc_roc: model.test_metrics.auc_roc,
		id: model.id,
		precision: default_threshold_test_metrics.precision,
		recall: default_threshold_test_metrics.recall,
	}
}

fn build_inner_multiclass_classifier(
	model: tangram_core::model::MulticlassClassifier,
) -> MulticlassClassifierProps {
	let classes = model.classes.to_owned();
	let class_metrics = model
		.test_metrics
		.class_metrics
		.iter()
		.map(|class_metrics| ClassMetrics {
			precision: class_metrics.precision,
			recall: class_metrics.recall,
		})
		.collect::<Vec<ClassMetrics>>();
	MulticlassClassifierProps {
		id: model.id.to_string(),
		accuracy: model.test_metrics.accuracy,
		baseline_accuracy: model.baseline_metrics.accuracy,
		class_metrics,
		classes,
	}
}
