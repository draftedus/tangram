use crate::app::{
	error::Error,
	pages::repos::new::actions::get_repo_for_model,
	types,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use serde::Serialize;
use tangram::id::Id;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingStatsViewModel {
	column_count: usize,
	column_stats: Vec<ColumnStats>,
	id: String,
	row_count: usize,
	target_column_stats: ColumnStats,
	title: String,
	repo: types::Repo,
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

pub async fn data(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
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
	let model = tangram::types::Model::from_slice(&data)?;

	let response = match model {
		tangram::types::Model::Classifier(model) => {
			let column_stats = model.overall_column_stats.as_option().unwrap();
			TrainingStatsViewModel {
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
				repo: get_repo_for_model(&db, model_id).await?,
			}
		}
		tangram::types::Model::Regressor(model) => {
			let column_stats = model.overall_column_stats.as_option().unwrap();
			TrainingStatsViewModel {
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
				repo: get_repo_for_model(&db, model_id).await?,
			}
		}
		_ => unimplemented!(),
	};
	let response = serde_json::to_vec(&response)?;

	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(response))?)
}

fn build_column_stats(column_stats: &tangram::types::ColumnStats) -> ColumnStats {
	match column_stats {
		tangram::types::ColumnStats::Unknown(column_stats) => ColumnStats {
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
		tangram::types::ColumnStats::Number(column_stats) => ColumnStats {
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
		tangram::types::ColumnStats::Enum(column_stats) => ColumnStats {
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
		tangram::types::ColumnStats::Text(column_stats) => ColumnStats {
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
