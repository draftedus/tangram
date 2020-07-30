use crate::app::{
	error::Error,
	pages::repos::new::actions::get_repo_for_model,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use std::collections::BTreeMap;
use tangram::id::Id;

pub async fn page(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, search_params).await?;
	let html = context
		.pinwheel
		.render("/repos/_repoId_/models/_modelId_", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
struct Props {
	id: String,
	inner: Inner,
	title: String,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "value")]
enum Inner {
	#[serde(rename = "BinaryClassifier")]
	BinaryClassifier(BinaryClassifier),
	#[serde(rename = "MulticlassClassifier")]
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(Serialize)]
struct BinaryClassifier {
	#[serde(rename = "classMetrics")]
	class_metrics: ClassMetrics,
	class: String,
	classes: Vec<String>,
	id: String,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MulticlassClassifier {
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
	let model = tangram::types::Model::from_slice(&data)?;
	// assemble the response
	let class = search_params.map(|s| s.get("class").unwrap().to_owned());
	let inner = match model {
		tangram::types::Model::Classifier(model) => match model.model.as_option().unwrap() {
			tangram::types::ClassificationModel::UnknownVariant(_, _, _) => unimplemented!(),
			tangram::types::ClassificationModel::LinearBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, id, class))
			}
			tangram::types::ClassificationModel::LinearMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, id, class))
			}
			tangram::types::ClassificationModel::GbtBinary(_) => {
				Inner::BinaryClassifier(build_inner_binary(model, id, class))
			}
			tangram::types::ClassificationModel::GbtMulticlass(_) => {
				Inner::MulticlassClassifier(build_inner_multiclass(model, id, class))
			}
		},
		_ => return Err(Error::BadRequest.into()),
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

fn build_inner_binary(
	model: tangram::types::Classifier,
	id: Id,
	class: Option<String>,
) -> BinaryClassifier {
	let test_metrics = model.test_metrics.as_option().unwrap();
	let class_metrics = test_metrics.class_metrics.as_option().unwrap();
	let classes = model.classes().to_owned();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		1
	};
	let class = class.unwrap_or_else(|| classes[class_index].to_owned());
	let class_metrics = &class_metrics[class_index];
	let class_metrics = ClassMetrics {
		precision: *class_metrics.precision.as_option().unwrap(),
		recall: *class_metrics.recall.as_option().unwrap(),
		f1_score: *class_metrics.f1_score.as_option().unwrap(),
		true_negatives: *class_metrics.true_negatives.as_option().unwrap(),
		true_positives: *class_metrics.true_positives.as_option().unwrap(),
		false_negatives: *class_metrics.false_negatives.as_option().unwrap(),
		false_positives: *class_metrics.false_positives.as_option().unwrap(),
	};
	BinaryClassifier {
		id: id.to_string(),
		class_metrics,
		classes,
		class,
	}
}

fn build_inner_multiclass(
	model: tangram::types::Classifier,
	id: Id,
	class: Option<String>,
) -> MulticlassClassifier {
	let test_metrics = model.test_metrics.as_option().unwrap();
	let classes = model.classes().to_owned();
	let class_metrics = test_metrics.class_metrics.as_option().unwrap();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		1
	};
	let class = class.unwrap_or_else(|| classes[class_index].to_owned());
	let class_metrics = &class_metrics[class_index];
	let class_metrics = ClassMetrics {
		precision: *class_metrics.precision.as_option().unwrap(),
		recall: *class_metrics.recall.as_option().unwrap(),
		f1_score: *class_metrics.f1_score.as_option().unwrap(),
		true_negatives: *class_metrics.true_negatives.as_option().unwrap(),
		true_positives: *class_metrics.true_positives.as_option().unwrap(),
		false_negatives: *class_metrics.false_negatives.as_option().unwrap(),
		false_positives: *class_metrics.false_positives.as_option().unwrap(),
	};
	MulticlassClassifier {
		id: id.to_string(),
		class_metrics,
		classes,
		class,
	}
}
