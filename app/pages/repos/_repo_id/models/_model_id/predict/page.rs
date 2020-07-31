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
) -> Result<Response<Body>> {
	let props = props(request, context, model_id).await?;
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_model_id/predict", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	columns: Vec<Column>,
	title: String,
	id: String,
	repo: types::Repo,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Column {
	Unknown(Unknown),
	Number(Number),
	Enum(Enum),
	Text(Text),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Unknown {
	name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Number {
	name: String,
	max: f32,
	min: f32,
}

#[derive(Serialize)]
struct Enum {
	name: String,
	options: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Text {
	name: String,
}

async fn props(request: Request<Body>, context: &Context, model_id: &str) -> Result<Props> {
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
	// get the necessary data from the model
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
	let row = rows.iter().next().ok_or(Error::NotFound)?;
	let id: Id = row.get(0);
	let title: String = row.get(1);
	let data: Vec<u8> = row.get(3);
	let model = tangram_core::types::Model::from_slice(&data)?;
	// assemble the response
	let column_stats = match model {
		tangram_core::types::Model::Classifier(model) => {
			model.overall_column_stats.into_option().unwrap()
		}
		tangram_core::types::Model::Regressor(model) => {
			model.overall_column_stats.into_option().unwrap()
		}
		_ => return Err(Error::BadRequest.into()),
	};
	let columns = column_stats
		.into_iter()
		.map(|column_stats| match column_stats {
			tangram_core::types::ColumnStats::Unknown(column_stats) => Column::Unknown(Unknown {
				name: column_stats.column_name.as_option().unwrap().to_owned(),
			}),
			tangram_core::types::ColumnStats::Number(column_stats) => Column::Number(Number {
				name: column_stats.column_name.as_option().unwrap().to_owned(),
				max: *column_stats.max.as_option().unwrap(),
				min: *column_stats.min.as_option().unwrap(),
			}),
			tangram_core::types::ColumnStats::Enum(column_stats) => {
				let histogram = column_stats.histogram.as_option().unwrap();
				let options = histogram.iter().map(|(key, _)| key.to_owned()).collect();
				Column::Enum(Enum {
					name: column_stats.column_name.as_option().unwrap().to_owned(),
					options,
				})
			}
			tangram_core::types::ColumnStats::Text(column_stats) => Column::Text(Text {
				name: column_stats.column_name.as_option().unwrap().to_owned(),
			}),
			tangram_core::types::ColumnStats::UnknownVariant(_, _, _) => unimplemented!(),
		})
		.collect();
	let repo = get_repo_for_model(&db, id).await?;
	db.commit().await?;

	Ok(Props {
		repo,
		id: id.to_string(),
		title,
		columns,
	})
}
