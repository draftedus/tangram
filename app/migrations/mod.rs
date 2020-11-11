use std::collections::BTreeMap;
use tangram_app_common::{sqlx, sqlx::prelude::*};
use tangram_util::error::Result;

fn migrations() -> BTreeMap<&'static str, &'static str> {
	let mut migrations: BTreeMap<&str, &str> = BTreeMap::new();
	migrations.insert(
		"2020-01-01-000000-init.sql",
		include_str!("./2020_01_01_000000_init.sql"),
	);
	migrations
}

pub async fn run(database_pool: &sqlx::AnyPool) -> Result<()> {
	let migrations = migrations();
	let mut db = database_pool.begin().await?;

	// Create the _migrations table if necessary.
	sqlx::query(
		"
			create table if not exists _migrations (
				name text primary key
			)
		",
	)
	.execute(&mut db)
	.await?;

	// Apply each migration in a transaction if it has not been applied already.
	for (name, sql) in migrations {
		let mut db = db.begin().await?;
		let migration_has_run: bool = sqlx::query(
			"
				select
					count(*) > 0
				from _migrations
				where
					name = $1
			",
		)
		.bind(&name)
		.fetch_one(&mut db)
		.await?
		.get(0);
		if !migration_has_run {
			db.execute(sql).await?;
			sqlx::query(
				"
					insert into _migrations (name) values ($1)
				",
			)
			.bind(&name)
			.execute(&mut db)
			.await?;
		}
		db.commit().await?;
	}

	db.commit().await?;
	Ok(())
}
