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
use num_traits::ToPrimitive;
use tangram::id::Id;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	column_count: usize,
	column_stats: Vec<ColumnStats>,
	id: String,
	row_count: usize,
	target_column_stats: ColumnStats,
	model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
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

#[derive(serde::Serialize)]
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
		.render_with("/repos/_repo_id/models/_model_id/training_stats/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
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

	let Model { data, .. } = get_model(&mut db, model_id).await?;
	let model = tangram::model::Model::from_slice(&data)?;

	let props = match model {
		tangram::model::Model::Classifier(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				id: model.id.to_owned(),
				row_count: model.row_count.to_usize().unwrap(),
				target_column_stats: build_column_stats(&model.overall_target_column_stats),
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				model_layout_info: get_model_layout_info(&mut db, model_id).await?,
			}
		}
		tangram::model::Model::Regressor(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				id: model.id.to_owned(),
				row_count: model.row_count.to_usize().unwrap(),
				target_column_stats: build_column_stats(&model.overall_target_column_stats),
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				model_layout_info: get_model_layout_info(&mut db, model_id).await?,
			}
		}
	};
	db.commit().await?;
	Ok(props)
}

fn build_column_stats(column_stats: &tangram::model::ColumnStats) -> ColumnStats {
	match column_stats {
		tangram::model::ColumnStats::Unknown(column_stats) => ColumnStats {
			column_type: ColumnType::Unknown,
			unique_count: None,
			invalid_count: None,
			name: column_stats.column_name.to_owned(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		tangram::model::ColumnStats::Number(column_stats) => ColumnStats {
			column_type: ColumnType::Number,
			unique_count: Some(column_stats.unique_count.to_usize().unwrap()),
			invalid_count: Some(column_stats.invalid_count.to_usize().unwrap()),
			name: column_stats.column_name.to_owned(),
			max: Some(column_stats.max),
			min: Some(column_stats.min),
			std: Some(column_stats.std),
			mean: Some(column_stats.mean),
			variance: Some(column_stats.variance),
		},
		tangram::model::ColumnStats::Enum(column_stats) => ColumnStats {
			column_type: ColumnType::Enum,
			unique_count: column_stats.unique_count.to_usize(),
			invalid_count: column_stats.invalid_count.to_usize(),
			name: column_stats.column_name.to_owned(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		tangram::model::ColumnStats::Text(column_stats) => ColumnStats {
			column_type: ColumnType::Text,
			unique_count: None,
			invalid_count: None,
			name: column_stats.column_name.to_owned(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
	}
}
