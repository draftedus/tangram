use crate::{
	error::Error,
	helpers::repos::get_repo_for_model,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	inner: Inner,
	title: String,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Regressor {
	baseline_mse: f32,
	baseline_rmse: f32,
	mse: f32,
	rmse: f32,
	id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BinaryClassifier {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
	losses: Vec<f32>,
	id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetrics {
	precision: f32,
	recall: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifier {
	accuracy: f32,
	baseline_accuracy: f32,
	class_metrics: Vec<ClassMetrics>,
	classes: Vec<String>,
	losses: Vec<f32>,
	id: String,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_modelId/training_metrics/", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	// get the necessary data from the model
	let rows = db
		.query(
			"
				select
					id,
					title,
					created_at,
					data
				from models
				where
					models.id = $1
			",
			&[&model_id],
		)
		.await?;
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let id: Id = row.get(0);
	let title: String = row.get(1);
	let data: Vec<u8> = row.get(3);
	let model = tangram_core::types::Model::from_slice(&data)?;
	// assemble the response
	let inner = match model {
		tangram_core::types::Model::Classifier(model) => match model.model.as_option().unwrap() {
			tangram_core::types::ClassificationModel::UnknownVariant(_, _, _) => unimplemented!(),
			tangram_core::types::ClassificationModel::LinearBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, id))
			}
			tangram_core::types::ClassificationModel::LinearMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, id))
			}
			tangram_core::types::ClassificationModel::GbtBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, id))
			}
			tangram_core::types::ClassificationModel::GbtMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, id))
			}
		},
		tangram_core::types::Model::Regressor(model) => {
			let test_metrics = model.test_metrics.as_option().unwrap();
			Inner::Regressor(Regressor {
				id: id.to_string(),
				rmse: *test_metrics.rmse.as_option().unwrap(),
				baseline_rmse: *test_metrics.baseline_rmse.as_option().unwrap(),
				mse: *test_metrics.mse.as_option().unwrap(),
				baseline_mse: *test_metrics.baseline_mse.as_option().unwrap(),
			})
		}
		_ => return Err(Error::NotFound.into()),
	};
	let repo = get_repo_for_model(&db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: id.to_string(),
		title,
		inner,
		repo,
	})
}

fn build_inner_binary(model: tangram_core::types::Classifier, id: Id) -> BinaryClassifier {
	let test_metrics = model.test_metrics.as_option().unwrap();
	let class_metrics = test_metrics.class_metrics.as_option().unwrap();
	let classes = model.classes().to_owned();
	let class_metrics = class_metrics
		.iter()
		.map(|class_metrics| ClassMetrics {
			precision: *class_metrics.precision.as_option().unwrap(),
			recall: *class_metrics.recall.as_option().unwrap(),
		})
		.collect::<Vec<ClassMetrics>>();
	let losses = match model.model.into_option().unwrap() {
		tangram_core::types::ClassificationModel::LinearBinary(inner_model) => {
			inner_model.losses.into_option().unwrap()
		}
		tangram_core::types::ClassificationModel::GbtBinary(inner_model) => {
			inner_model.losses.into_option().unwrap()
		}
		_ => unreachable!(),
	};
	BinaryClassifier {
		id: id.to_string(),
		accuracy: *test_metrics.accuracy.as_option().unwrap(),
		baseline_accuracy: *test_metrics.baseline_accuracy.as_option().unwrap(),
		class_metrics,
		classes,
		losses,
	}
}

fn build_inner_multiclass(model: tangram_core::types::Classifier, id: Id) -> MulticlassClassifier {
	let test_metrics = model.test_metrics.as_option().unwrap();
	let classes = model.classes().to_owned();
	let class_metrics = test_metrics.class_metrics.as_option().unwrap();
	let class_metrics = class_metrics
		.iter()
		.map(|class_metrics| ClassMetrics {
			precision: *class_metrics.precision.as_option().unwrap(),
			recall: *class_metrics.recall.as_option().unwrap(),
		})
		.collect::<Vec<ClassMetrics>>();
	let losses = match model.model.into_option().unwrap() {
		tangram_core::types::ClassificationModel::LinearMulticlass(inner_model) => {
			inner_model.losses.into_option().unwrap()
		}
		tangram_core::types::ClassificationModel::GbtMulticlass(inner_model) => {
			inner_model.losses.into_option().unwrap()
		}
		_ => unreachable!(),
	};
	MulticlassClassifier {
		id: id.to_string(),
		accuracy: *test_metrics.accuracy.as_option().unwrap(),
		baseline_accuracy: *test_metrics.baseline_accuracy.as_option().unwrap(),
		class_metrics,
		classes,
		losses,
	}
}
