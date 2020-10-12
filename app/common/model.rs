use anyhow::Result;
use sqlx::prelude::*;
use tangram_util::id::Id;

/// Retrieves the model with the specified id.
pub async fn get_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<tangram_core::model::Model> {
	let row = sqlx::query(
		"
			select
				data
			from models
			where
				models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let data: String = row.get(0);
	let data: Vec<u8> = base64::decode(data)?;
	let model = tangram_core::model::Model::from_slice(&data.as_slice())?;
	Ok(model)
}
