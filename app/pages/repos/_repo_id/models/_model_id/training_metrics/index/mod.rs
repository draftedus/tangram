use crate::{
	common::{
		error::Error,
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	Regressor(RegressorInner),
	BinaryClassifier(BinaryClassifierInner),
	MulticlassClassifier(MulticlassClassifierInner),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegressorInner {
	baseline_mse: f32,
	baseline_rmse: f32,
	mse: f32,
	rmse: f32,
	id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BinaryClassifierInner {
	auc_roc: f32,
	id: String,
	losses: Option<Vec<f32>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifierInner {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
	id: String,
	losses: Option<Vec<f32>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetrics {
	precision: f32,
	recall: f32,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/training_metrics/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
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
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}

fn build_inner_regressor(model: tangram_core::model::Regressor) -> RegressorInner {
	RegressorInner {
		id: model.id,
		rmse: model.test_metrics.rmse,
		baseline_rmse: model.test_metrics.baseline_rmse,
		mse: model.test_metrics.mse,
		baseline_mse: model.test_metrics.baseline_mse,
	}
}

fn build_inner_binary_classifier(
	model: tangram_core::model::BinaryClassifier,
) -> BinaryClassifierInner {
	let losses = match model.model {
		tangram_core::model::BinaryClassificationModel::Linear(model) => model.losses,
		tangram_core::model::BinaryClassificationModel::Tree(model) => model.losses,
	};
	BinaryClassifierInner {
		id: model.id,
		auc_roc: model.test_metrics.auc_roc,
		losses,
	}
}

fn build_inner_multiclass_classifier(
	model: tangram_core::model::MulticlassClassifier,
) -> MulticlassClassifierInner {
	let test_metrics = &model.test_metrics;
	let classes = model.classes().to_owned();
	let class_metrics = &test_metrics.class_metrics;
	let class_metrics = class_metrics
		.iter()
		.map(|class_metrics| ClassMetrics {
			precision: class_metrics.precision,
			recall: class_metrics.recall,
		})
		.collect::<Vec<ClassMetrics>>();
	let losses = match model.model {
		tangram_core::model::MulticlassClassificationModel::Linear(model) => model.losses,
		tangram_core::model::MulticlassClassificationModel::Tree(model) => model.losses,
	};
	MulticlassClassifierInner {
		id: model.id.to_string(),
		accuracy: test_metrics.accuracy,
		baseline_accuracy: test_metrics.baseline_accuracy,
		class_metrics,
		classes,
		losses,
	}
}
