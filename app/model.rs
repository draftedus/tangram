use anyhow::Result;
use tangram::{id::Id, types};
use tokio_postgres as postgres;

/// Retrieves the model with the specified id. Errors if the model is not found.
pub async fn get_model(db: &postgres::Transaction<'_>, model_id: Id) -> Result<types::Model> {
	let data: Vec<u8> = db
		.query_one(
			"
				select
					data
				from models
				where
					models.id = $1
			",
			&[&model_id.to_string()],
		)
		.await?
		.get(0);
	let model = types::Model::from_slice(&data)?;
	Ok(model)
}
