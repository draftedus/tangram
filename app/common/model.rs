use anyhow::Result;
use sqlx::prelude::*;
use tangram_core::util::id::Id;

pub struct Model {
	pub id: Id,
	pub data: Vec<u8>,
}

pub async fn get_model(db: &mut sqlx::Transaction<'_, sqlx::Any>, model_id: Id) -> Result<Model> {
	let row = sqlx::query(
		"
			select
				id,
				created_at,
				data
			from models
			where
				models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let id: String = row.get(0);
	let id: Id = id.parse()?;
	let data: String = row.get(2);
	let data: Vec<u8> = base64::decode(data)?;
	Ok(Model { id, data })
}
