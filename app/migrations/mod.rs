use anyhow::Result;
use deadpool_postgres::Pool;
use std::collections::BTreeMap;

fn migrations() -> BTreeMap<&'static str, &'static str> {
	let mut migrations: BTreeMap<&str, &str> = BTreeMap::new();
	migrations.insert(
		"2020-01-01-000000-init.sql",
		include_str!("./2020-01-01-000000-init.sql"),
	);
	migrations
}

pub async fn run(database_pool: &Pool) -> Result<()> {
	let migrations = migrations();
	let mut db = database_pool.get().await?;

	// create the _migrations table if necessary
	let has_migrations: bool = db
		.query_one(
			"
        select
          count(*) > 0
        from information_schema.tables
        where
          table_name = '_migrations'
      ",
			&[],
		)
		.await?
		.get(0);
	if !has_migrations {
		db.execute("create table _migrations ( name text primary key )", &[])
			.await?;
	}

	// apply each migration in a transaction if it has not been applied already
	for (name, sql) in migrations.iter() {
		let db = db.transaction().await?;
		let migration_has_run: bool = db
			.query_one(
				"select count(*) > 0 from _migrations where name = $1",
				&[name],
			)
			.await?
			.get(0);
		if !migration_has_run {
			db.batch_execute(sql).await?;
			db.execute("insert into _migrations (name) values ($1)", &[name])
				.await?;
		}
		db.commit().await?;
	}

	Ok(())
}
