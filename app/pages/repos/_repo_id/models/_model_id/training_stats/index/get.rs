use super::render::{
	render, ColumnStatsTableProps, ColumnStatsTableRow, ColumnType, Props,
	TargetColumnStatsTableProps,
};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{document::PageInfo, model_layout::get_model_layout_info};
use tangram_deps::{http, hyper, num_traits::ToPrimitive};
use tangram_util::{error::Result, id::Id};

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
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
	let props = match model {
		tangram_core::model::Model::Regressor(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				column_stats_table: ColumnStatsTableProps {
					column_stats_table_rows: column_stats
						.iter()
						.map(|column_stats| build_column_stats(column_stats))
						.collect(),
				},
				id: model.id.clone(),
				model_layout_info: get_model_layout_info(&mut db, context, model_id).await?,
				column_count: column_stats.len(),
				row_count: model.test_row_count.to_usize().unwrap()
					+ model.train_row_count.to_usize().unwrap(),
				target_column_stats_table: TargetColumnStatsTableProps {
					target_column_stats_table_row: build_column_stats(
						&model.overall_target_column_stats,
					),
				},
			}
		}
		tangram_core::model::Model::BinaryClassifier(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				column_stats_table: ColumnStatsTableProps {
					column_stats_table_rows: column_stats
						.iter()
						.map(|column_stats| build_column_stats(column_stats))
						.collect(),
				},
				id: model.id.clone(),
				model_layout_info: get_model_layout_info(&mut db, context, model_id).await?,
				column_count: column_stats.len(),
				row_count: model.test_row_count.to_usize().unwrap()
					+ model.train_row_count.to_usize().unwrap(),
				target_column_stats_table: TargetColumnStatsTableProps {
					target_column_stats_table_row: build_column_stats(
						&model.overall_target_column_stats,
					),
				},
			}
		}
		tangram_core::model::Model::MulticlassClassifier(model) => {
			let column_stats = model.overall_column_stats;
			Props {
				column_stats_table: ColumnStatsTableProps {
					column_stats_table_rows: column_stats
						.iter()
						.map(|column_stats| build_column_stats(column_stats))
						.collect(),
				},
				id: model.id.clone(),
				model_layout_info: get_model_layout_info(&mut db, context, model_id).await?,
				row_count: model.test_row_count.to_usize().unwrap()
					+ model.train_row_count.to_usize().unwrap(),
				column_count: column_stats.len(),
				target_column_stats_table: TargetColumnStatsTableProps {
					target_column_stats_table_row: build_column_stats(
						&model.overall_target_column_stats,
					),
				},
			}
		}
	};
	db.commit().await?;
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn build_column_stats(column_stats: &tangram_core::model::ColumnStats) -> ColumnStatsTableRow {
	match column_stats {
		tangram_core::model::ColumnStats::Unknown(column_stats) => ColumnStatsTableRow {
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
		tangram_core::model::ColumnStats::Number(column_stats) => ColumnStatsTableRow {
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
		tangram_core::model::ColumnStats::Enum(column_stats) => ColumnStatsTableRow {
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
		tangram_core::model::ColumnStats::Text(column_stats) => ColumnStatsTableRow {
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
