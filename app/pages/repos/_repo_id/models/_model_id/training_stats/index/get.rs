use super::props::{ColumnStats, ColumnType, Props};
use crate::{
	common::{
		error::Error,
		model::get_model,
		user::{authorize_user, authorize_user_for_model},
	},
	layouts::model_layout::get_model_layout_info,
	Context,
};
use hyper::{Body, Request, Response, StatusCode};
use num_traits::ToPrimitive;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/_model_id/training_stats/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

pub async fn props(context: &Context, request: Request<Body>, model_id: &str) -> Result<Props> {
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
	let props = match model {
		tangram_core::model::Model::Regressor(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				id: model.id.clone(),
				model_layout_info: get_model_layout_info(&mut db, context, model_id).await?,
				target_column_stats: build_column_stats(&model.overall_target_column_stats),
				row_count: model.test_row_count.to_usize().unwrap()
					+ model.train_row_count.to_usize().unwrap(),
			}
		}
		tangram_core::model::Model::BinaryClassifier(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				id: model.id.clone(),
				model_layout_info: get_model_layout_info(&mut db, context, model_id).await?,
				target_column_stats: build_column_stats(&model.overall_target_column_stats),
				row_count: model.test_row_count.to_usize().unwrap()
					+ model.train_row_count.to_usize().unwrap(),
			}
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				column_count: column_stats.len(),
				column_stats: column_stats
					.iter()
					.map(|column_stats| build_column_stats(column_stats))
					.collect(),
				id: model.id.clone(),
				model_layout_info: get_model_layout_info(&mut db, context, model_id).await?,
				target_column_stats: build_column_stats(&model.overall_target_column_stats),
				row_count: model.test_row_count.to_usize().unwrap()
					+ model.train_row_count.to_usize().unwrap(),
			}
		}
	};
	db.commit().await?;
	Ok(props)
}

fn build_column_stats(column_stats: &tangram_core::model::ColumnStats) -> ColumnStats {
	match column_stats {
		tangram_core::model::ColumnStats::Unknown(column_stats) => ColumnStats {
			column_type: ColumnType::Unknown,
			unique_count: None,
			invalid_count: None,
			name: column_stats.column_name.clone(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		tangram_core::model::ColumnStats::Number(column_stats) => ColumnStats {
			column_type: ColumnType::Number,
			unique_count: Some(column_stats.unique_count.to_usize().unwrap()),
			invalid_count: Some(column_stats.invalid_count.to_usize().unwrap()),
			name: column_stats.column_name.clone(),
			max: Some(column_stats.max),
			min: Some(column_stats.min),
			std: Some(column_stats.std),
			mean: Some(column_stats.mean),
			variance: Some(column_stats.variance),
		},
		tangram_core::model::ColumnStats::Enum(column_stats) => ColumnStats {
			column_type: ColumnType::Enum,
			unique_count: column_stats.unique_count.to_usize(),
			invalid_count: column_stats.invalid_count.to_usize(),
			name: column_stats.column_name.clone(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
		tangram_core::model::ColumnStats::Text(column_stats) => ColumnStats {
			column_type: ColumnType::Text,
			unique_count: None,
			invalid_count: None,
			name: column_stats.column_name.clone(),
			max: None,
			min: None,
			std: None,
			mean: None,
			variance: None,
		},
	}
}
