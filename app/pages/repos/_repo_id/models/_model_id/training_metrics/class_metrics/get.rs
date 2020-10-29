use super::props::*;
use crate::{
	common::{
		error::Error,
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id, search_params).await?;
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

pub async fn props(
	context: &Context,
	request: Request<Body>,
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
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let model = get_model(&mut db, model_id).await?;
	let model = match model {
		tangram_core::model::Model::MulticlassClassifier(model) => model,
		_ => return Err(Error::BadRequest.into()),
	};
	let class = search_params.map(|s| s.get("class").unwrap().to_owned());
	let classes = model.classes.to_owned();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		0
	};
	let class = class.unwrap_or_else(|| classes[class_index].to_owned());
	let class_metrics = &model.test_metrics.class_metrics[class_index];
	let precision = class_metrics.precision;
	let recall = class_metrics.recall;
	let f1_score = class_metrics.f1_score;
	let true_negatives = class_metrics.true_negatives;
	let true_positives = class_metrics.true_positives;
	let false_negatives = class_metrics.false_negatives;
	let false_positives = class_metrics.false_positives;
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		model_layout_info,
		class,
		classes,
		f1_score,
		false_negatives,
		false_positives,
		precision,
		recall,
		true_negatives,
		true_positives,
	})
}
