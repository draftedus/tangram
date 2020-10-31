use super::props::{Enum, Inner, Number, Props, Text, TokenStats};
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
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	column_name: &str,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id, column_name).await?;
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

pub async fn props(
	context: &Context,
	request: Request<Body>,
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
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let model = get_model(&mut db, model_id).await?;
	let (mut column_stats, target_column_stats) = match model {
		tangram_core::model::Model::Regressor(model) => (
			model.overall_column_stats,
			model.overall_target_column_stats,
		),
		tangram_core::model::Model::BinaryClassifier(model) => (
			model.overall_column_stats,
			model.overall_target_column_stats,
		),
		tangram_core::model::Model::MulticlassClassifier(model) => (
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
		tangram_core::model::ColumnStats::Unknown(_) => todo!(),
		tangram_core::model::ColumnStats::Number(column) => Inner::Number(Number {
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
					token: token.token.to_string(),
					count: token.occurrence_count,
					examples_count: token.examples_count,
				})
				.collect(),
		}),
	};
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	db.commit().await?;
	Ok(Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	})
}
