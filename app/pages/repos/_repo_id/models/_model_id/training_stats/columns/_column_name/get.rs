use super::props::{Enum, Inner, Number, Props, Text, TokenStats};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_deps::{http, hyper, pinwheel::Pinwheel};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	pinwheel: &Pinwheel,
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	column_name: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
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
		return Ok(not_found());
	};
	let inner = match column {
		tangram_core::model::ColumnStats::Unknown(_) => todo!(),
		tangram_core::model::ColumnStats::Number(column) => Inner::Number(Number {
			invalid_count: column.invalid_count,
			min: column.min,
			max: column.max,
			mean: column.mean,
			name: column.column_name.clone(),
			p25: column.p25,
			p50: column.p50,
			p75: column.p75,
			std: column.std,
			unique_count: column.unique_count,
		}),
		tangram_core::model::ColumnStats::Enum(column) => Inner::Enum(Enum {
			histogram: Some(column.histogram),
			invalid_count: column.invalid_count,
			name: column.column_name.clone(),
			unique_count: column.unique_count,
		}),
		tangram_core::model::ColumnStats::Text(column) => Inner::Text(Text {
			name: column.column_name.clone(),
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
	let props = Props {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	};
	db.commit().await?;
	let html = pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_stats/columns/_column_name",
		props,
	)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
