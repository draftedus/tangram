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
use tangram_id::Id;

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
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Regressor {
	baseline_mse: f32,
	baseline_rmse: f32,
	mse: f32,
	rmse: f32,
	id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BinaryClassifier {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
	losses: Vec<f32>,
	id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetrics {
	precision: f32,
	recall: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifier {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
	losses: Vec<f32>,
	id: String,
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
		tangram_core::model::Model::Classifier(model) => match model.model {
			tangram_core::model::ClassificationModel::LinearBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, model_id))
			}
			tangram_core::model::ClassificationModel::LinearMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, model_id))
			}
			tangram_core::model::ClassificationModel::TreeBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, model_id))
			}
			tangram_core::model::ClassificationModel::TreeMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, model_id))
			}
		},
		tangram_core::model::Model::Regressor(model) => {
			let test_metrics = model.test_metrics;
			Inner::Regressor(Regressor {
				id: model_id.to_string(),
				rmse: test_metrics.rmse,
				baseline_rmse: test_metrics.baseline_rmse,
				mse: test_metrics.mse,
				baseline_mse: test_metrics.baseline_mse,
			})
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

fn build_inner_binary(model: tangram_core::model::Classifier, id: Id) -> BinaryClassifier {
	let test_metrics = &model.test_metrics;
	let class_metrics = &test_metrics.class_metrics;
	let classes = model.classes().to_owned();
	let class_metrics = class_metrics
		.iter()
		.map(|class_metrics| ClassMetrics {
			precision: class_metrics.precision,
			recall: class_metrics.recall,
		})
		.collect::<Vec<ClassMetrics>>();
	let losses = match model.model {
		tangram_core::model::ClassificationModel::LinearBinary(inner_model) => inner_model.losses,
		tangram_core::model::ClassificationModel::TreeBinary(inner_model) => inner_model.losses,
		_ => unreachable!(),
	};
	BinaryClassifier {
		id: id.to_string(),
		accuracy: test_metrics.accuracy,
		baseline_accuracy: test_metrics.baseline_accuracy,
		class_metrics,
		classes,
		losses,
	}
}

fn build_inner_multiclass(model: tangram_core::model::Classifier, id: Id) -> MulticlassClassifier {
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
		tangram_core::model::ClassificationModel::LinearMulticlass(inner_model) => {
			inner_model.losses
		}
		tangram_core::model::ClassificationModel::TreeMulticlass(inner_model) => inner_model.losses,
		_ => unreachable!(),
	};
	MulticlassClassifier {
		id: id.to_string(),
		accuracy: test_metrics.accuracy,
		baseline_accuracy: test_metrics.baseline_accuracy,
		class_metrics,
		classes,
		losses,
	}
}
