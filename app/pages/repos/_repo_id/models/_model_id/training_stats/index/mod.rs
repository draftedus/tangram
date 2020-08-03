use crate::{
	error::Error,
	helpers::repos::get_model_layout_props,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use serde::Serialize;
use sqlx::prelude::*;
use tangram_core::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	column_count: usize,
	column_stats: Vec<ColumnStats>,
	id: String,
	row_count: usize,
	target_column_stats: ColumnStats,
	title: String,
	model_layout_props: types::ModelLayoutProps,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ColumnStats {
	invalid_count: Option<usize>,
	max: Option<f32>,
	mean: Option<f32>,
	min: Option<f32>,
	name: String,
	std: Option<f32>,
	column_type: ColumnType,
	unique_count: Option<usize>,
	variance: Option<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_model_id/training_stats/", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let rows = sqlx::query(
		"
			select
				id,
				title,
				created_at,
				data
			from models
			where
				models.id = ?1
			",
	)
	.bind(&model_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	// TODO error handling
	let row = rows.iter().next().unwrap();
	let title: String = row.get(1);
	let data: String = row.get(3);
	let data: Vec<u8> = base64::decode(data).unwrap();
	let model = tangram_core::types::Model::from_slice(&data)?;

	let props = match model {
		tangram_core::types::Model::Classifier(model) => {
			let column_stats = model.overall_column_stats.as_option().unwrap();
			Props {
				id: model.id.as_option().unwrap().to_owned(),
				row_count: model.row_count.as_option().unwrap().to_usize().unwrap(),
				target_column_stats: build_column_stats(
					model.overall_target_column_stats.as_option().unwrap(),
				),
				title,
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				model_layout_props: get_model_layout_props(&mut db, model_id).await?,
			}
		}
		tangram_core::types::Model::Regressor(model) => {
			let column_stats = model.overall_column_stats.as_option().unwrap();
			Props {
				id: model.id.as_option().unwrap().to_owned(),
				row_count: model.row_count.as_option().unwrap().to_usize().unwrap(),
				target_column_stats: build_column_stats(
					model.overall_target_column_stats.as_option().unwrap(),
				),
				title,
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				model_layout_props: get_model_layout_props(&mut db, model_id).await?,
			}
		}
		_ => unimplemented!(),
	};
	db.commit().await?;
	Ok(props)
}

fn build_column_stats(column_stats: &tangram_core::types::ColumnStats) -> ColumnStats {
	match column_stats {
		tangram_core::types::ColumnStats::Unknown(column_stats) => ColumnStats {
			column_type: ColumnType::Unknown,
			unique_count: None,
			invalid_count: None,
			name: column_stats.column_name.as_option().unwrap().to_owned(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		tangram_core::types::ColumnStats::Number(column_stats) => ColumnStats {
			column_type: ColumnType::Number,
			unique_count: Some(
				column_stats
					.unique_count
					.as_option()
					.unwrap()
					.to_usize()
					.unwrap(),
			),
			invalid_count: Some(
				column_stats
					.invalid_count
					.as_option()
					.unwrap()
					.to_usize()
					.unwrap(),
			),
			name: column_stats.column_name.as_option().unwrap().to_owned(),
			max: Some(*column_stats.max.as_option().unwrap()),
			min: Some(*column_stats.min.as_option().unwrap()),
			std: Some(*column_stats.std.as_option().unwrap()),
			mean: Some(*column_stats.mean.as_option().unwrap()),
			variance: Some(*column_stats.variance.as_option().unwrap()),
		},
		tangram_core::types::ColumnStats::Enum(column_stats) => ColumnStats {
			column_type: ColumnType::Enum,
			unique_count: column_stats.unique_count.as_option().unwrap().to_usize(),
			invalid_count: column_stats.invalid_count.as_option().unwrap().to_usize(),
			name: column_stats.column_name.as_option().unwrap().to_owned(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		tangram_core::types::ColumnStats::Text(column_stats) => ColumnStats {
			column_type: ColumnType::Text,
			unique_count: None,
			invalid_count: None,
			name: column_stats.column_name.as_option().unwrap().to_owned(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		_ => unimplemented!(),
	}
}
