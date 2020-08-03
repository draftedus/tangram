use anyhow::Result;
use sqlx::prelude::*;
use tangram_core::{id::Id, types};

/// Retrieves the model with the specified id. Errors if the model is not found.
pub async fn get_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<types::Model> {
	let data: String = sqlx::query(
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
	.await?
	.get(0);
	let data: Vec<u8> = base64::decode(data)?;
	let model = types::Model::from_slice(&data.as_slice())?;
	Ok(model)
}
