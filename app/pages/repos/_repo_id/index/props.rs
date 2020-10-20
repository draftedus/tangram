use anyhow::Result;
use chrono::prelude::*;
use sqlx::prelude::*;
use tangram_util::id::Id;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	models: Vec<Model>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Model {
	id: String,
	created_at: String,
}

pub async fn props(db: &mut sqlx::Transaction<'_, sqlx::Any>, repo_id: Id) -> Result<Props> {
	let rows = sqlx::query(
		"
			select
				models.id,
				models.created_at
			from models
			where models.repo_id = ?1
			order by models.created_at
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(db)
	.await?;
	let models = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let id: Id = id.parse().unwrap();
			let created_at: i64 = row.get(1);
			let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
			Model {
				created_at: created_at.to_rfc3339(),
				id: id.to_string(),
			}
		})
		.collect();
	Ok(Props { models })
}
