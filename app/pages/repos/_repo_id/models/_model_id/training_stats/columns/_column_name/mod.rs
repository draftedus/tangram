use crate::{
	common::{
		model::get_model,
		model_layout_info::{get_model_layout_info, ModelLayoutInfo},
		user::{authorize_user, authorize_user_for_model},
	},
	error::Error,
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use tangram_id::Id;

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
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Number {
	histogram: Option<Vec<(f32, u64)>>,
	invalid_count: u64,
	max: f32,
	mean: f32,
	min: f32,
	name: String,
	p25: f32,
	p50: f32,
	p75: f32,
	std: f32,
	unique_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Enum {
	histogram: Option<Vec<(String, u64)>>,
	invalid_count: u64,
	name: String,
	unique_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenStats {
	token: String,
	count: u64,
	examples_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Text {
	name: String,
	tokens: Vec<TokenStats>,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	column_name: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, column_name).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_stats/columns/_column_name",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	column_name: &str,
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
	let model = get_model(&mut db, model_id).await?;
	let (mut column_stats, target_column_stats) = match model {
		tangram_core::model::Model::Classifier(model) => (
			model.overall_column_stats,
			model.overall_target_column_stats,
		),
		tangram_core::model::Model::Regressor(model) => (
			model.overall_column_stats,
			model.overall_target_column_stats,
		),
	};

	let column_index = column_stats
		.iter()
		.position(|column_stats| column_stats.column_name() == column_name);

	let column = if target_column_stats.column_name() == column_name {
		target_column_stats
	} else if let Some(column_index) = column_index {
		column_stats
			.drain(column_index..column_index + 1)
			.next()
			.unwrap()
	} else {
		return Err(Error::NotFound.into());
	};

	let inner = match column {
		tangram_core::model::ColumnStats::Unknown(_) => unimplemented!(),
		tangram_core::model::ColumnStats::Number(column) => Inner::Number(Number {
			histogram: column.histogram,
			invalid_count: column.invalid_count.to_owned(),
			min: column.min,
			max: column.max,
			mean: column.mean,
			name: column.column_name.to_owned(),
			p25: column.p25,
			p50: column.p50,
			p75: column.p75,
			std: column.std,
			unique_count: column.unique_count,
		}),
		tangram_core::model::ColumnStats::Enum(column) => Inner::Enum(Enum {
			histogram: Some(column.histogram),
			invalid_count: column.invalid_count.to_owned(),
			name: column.column_name.to_owned(),
			unique_count: column.unique_count,
		}),
		tangram_core::model::ColumnStats::Text(column) => Inner::Text(Text {
			name: column.column_name.to_owned(),
			tokens: column
				.top_tokens
				.into_iter()
				.map(|token| TokenStats {
					token: token.token,
					count: token.count,
					examples_count: token.examples_count,
				})
				.collect(),
		}),
	};

	let model_layout_info = get_model_layout_info(&mut db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}
