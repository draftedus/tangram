use crate::{
	error::Error,
	pages::repos::new::actions::get_repo_for_model,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use tangram_core::id::Id;

pub async fn page(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	column_name: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id, column_name).await?;
	let html = context
		.pinwheel
		.render(
			"/repos/_repo_id/models/_model_id/training_stats/columns/_column_name",
			props,
		)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	id: String,
	title: String,
	inner: Inner,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
enum Inner {
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Enum {
	histogram: Option<Vec<(String, u64)>>,
	invalid_count: u64,
	name: String,
	unique_count: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Text {
	name: String,
	tokens: Vec<(String, u64)>,
}

async fn props(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	column_name: &str,
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
	// TODO error handling
	let row = rows.iter().next().unwrap();
	let title: String = row.get(1);
	let data: Vec<u8> = row.get(3);
	let model = tangram_core::types::Model::from_slice(&data)?;

	let (mut column_stats, target_column_stats) = match model {
		tangram_core::types::Model::Classifier(model) => (
			model.overall_column_stats.into_option().unwrap(),
			model.overall_target_column_stats.into_option().unwrap(),
		),
		tangram_core::types::Model::Regressor(model) => (
			model.overall_column_stats.into_option().unwrap(),
			model.overall_target_column_stats.into_option().unwrap(),
		),
		_ => unimplemented!(),
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
		tangram_core::types::ColumnStats::UnknownVariant(_, _, _) => unimplemented!(),
		tangram_core::types::ColumnStats::Unknown(_) => unimplemented!(),
		tangram_core::types::ColumnStats::Number(column) => Inner::Number(Number {
			histogram: column.histogram.into_option().unwrap(),
			invalid_count: column.invalid_count.as_option().unwrap().to_owned(),
			min: *column.min.as_option().unwrap(),
			max: *column.max.as_option().unwrap(),
			mean: *column.mean.as_option().unwrap(),
			name: column.column_name.as_option().unwrap().to_owned(),
			p25: *column.p25.as_option().unwrap(),
			p50: *column.p50.as_option().unwrap(),
			p75: *column.p75.as_option().unwrap(),
			std: *column.std.as_option().unwrap(),
			unique_count: *column.unique_count.as_option().unwrap(),
		}),
		tangram_core::types::ColumnStats::Enum(column) => Inner::Enum(Enum {
			histogram: column.histogram.into_option(),
			invalid_count: column.invalid_count.as_option().unwrap().to_owned(),
			name: column.column_name.as_option().unwrap().to_owned(),
			unique_count: *column.unique_count.as_option().unwrap(),
		}),
		tangram_core::types::ColumnStats::Text(column) => Inner::Text(Text {
			name: column.column_name.as_option().unwrap().to_owned(),
			tokens: column.top_tokens.into_option().unwrap(),
		}),
	};

	let repo = get_repo_for_model(&db, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		title,
		inner,
		repo,
	})
}
