use crate::app::{
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
use tangram::id::Id;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let class = search_params.map(|s| s.get("class").unwrap().to_owned());
	let props = props(request, context, model_id, class).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_metrics/precision_recall",
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
	class: String,
	classes: Vec<String>,
	data: Vec<DataPoint>,
	id: String,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DataPoint {
	precision: f32,
	recall: f32,
	threshold: f32,
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	class: Option<String>,
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
	let model = tangram::model::Model::from_slice(&data)?;
	let model = match model {
		tangram::model::Model::Classifier(model) => model,
		_ => return Err(Error::BadRequest.into()),
	};
	let classes = model.classes().to_owned();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		1
	};
	let class = class.unwrap_or_else(|| classes[class_index].to_owned());
	let class_metrics = match &model.model {
		tangram::model::ClassificationModel::LinearBinary(inner_model) => {
			&inner_model.class_metrics
		}
		tangram::model::ClassificationModel::TreeBinary(inner_model) => &inner_model.class_metrics,
		_ => return Err(Error::BadRequest.into()),
	};
	let data = class_metrics
		.get(class_index)
		.unwrap()
		.thresholds
		.iter()
		.map(|class_metrics| DataPoint {
			precision: class_metrics.precision,
			recall: class_metrics.recall,
			threshold: class_metrics.threshold,
		})
		.collect();
	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		class,
		classes: model.classes().to_owned(),
		data,
		id: id.to_string(),
		model_layout_info,
	})
}
