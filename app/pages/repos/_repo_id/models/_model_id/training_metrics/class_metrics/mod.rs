use crate::{
	common::{
		model::{get_model, Model},
		repos::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_core::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, search_params).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_metrics/class_metrics",
		props,
	)?;
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
#[serde(tag = "type", content = "value")]
enum Inner {
	#[serde(rename = "BinaryClassifier")]
	BinaryClassifier(BinaryClassifier),
	#[serde(rename = "MulticlassClassifier")]
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(serde::Serialize)]
struct BinaryClassifier {
	#[serde(rename = "classMetrics")]
	class_metrics: ClassMetrics,
	class: String,
	classes: Vec<String>,
	id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassMetrics {
	precision: f32,
	recall: f32,
	f1_score: f32,
	false_negatives: u64,
	false_positives: u64,
	true_positives: u64,
	true_negatives: u64,
}

#[derive(serde::Serialize)]
struct MulticlassClassifier {
	#[serde(rename = "classMetrics")]
	class_metrics: ClassMetrics,
	classes: Vec<String>,
	id: String,
	class: String,
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Props> {
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
	let Model { data, id } = get_model(&mut db, model_id).await?;
	let model = tangram_core::types::Model::from_slice(&data)?;
	// assemble the response
	let class = search_params.map(|s| s.get("class").unwrap().to_owned());
	let inner = match model {
		tangram_core::types::Model::Classifier(model) => match model.model {
			tangram_core::types::ClassificationModel::LinearBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, id, class))
			}
			tangram_core::types::ClassificationModel::LinearMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, id, class))
			}
			tangram_core::types::ClassificationModel::GbtBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, id, class))
			}
			tangram_core::types::ClassificationModel::GbtMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, id, class))
			}
		},
		_ => return Err(Error::BadRequest.into()),
	};
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: id.to_string(),
		inner,
		model_layout_info,
	})
}

fn build_inner_binary(
	model: tangram_core::types::Classifier,
	id: Id,
	class: Option<String>,
) -> BinaryClassifier {
	let test_metrics = &model.test_metrics;
	let class_metrics = &test_metrics.class_metrics;
	let classes = model.classes().to_owned();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		1
	};
	let class = class.unwrap_or_else(|| classes[class_index].to_owned());
	let class_metrics = &class_metrics[class_index];
	let class_metrics = ClassMetrics {
		precision: class_metrics.precision,
		recall: class_metrics.recall,
		f1_score: class_metrics.f1_score,
		true_negatives: class_metrics.true_negatives,
		true_positives: class_metrics.true_positives,
		false_negatives: class_metrics.false_negatives,
		false_positives: class_metrics.false_positives,
	};
	BinaryClassifier {
		id: id.to_string(),
		class_metrics,
		classes,
		class,
	}
}

fn build_inner_multiclass(
	model: tangram_core::types::Classifier,
	id: Id,
	class: Option<String>,
) -> MulticlassClassifier {
	let test_metrics = &model.test_metrics;
	let classes = model.classes().to_owned();
	let class_metrics = &test_metrics.class_metrics;
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		0
	};
	let class = class.unwrap_or_else(|| classes[class_index].to_owned());

	let class_metrics = &class_metrics[class_index];
	let class_metrics = ClassMetrics {
		precision: class_metrics.precision,
		recall: class_metrics.recall,
		f1_score: class_metrics.f1_score,
		true_negatives: class_metrics.true_negatives,
		true_positives: class_metrics.true_positives,
		false_negatives: class_metrics.false_negatives,
		false_positives: class_metrics.false_positives,
	};
	MulticlassClassifier {
		id: id.to_string(),
		class_metrics,
		classes,
		class,
	}
}
