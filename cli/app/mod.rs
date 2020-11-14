use self::license::verify_license;
use crate::AppArgs;
use std::path::PathBuf;
use tangram_util::{err, error::Result};
use url::Url;

mod license;

#[cfg(feature = "app")]
pub(crate) fn app(args: AppArgs) -> Result<()> {
	// Verify the license if one was provided.
	let license_verified: Option<bool> = if let Some(license_file_path) = args.license {
		Some(verify_license(&license_file_path)?)
	} else {
		None
	};
	// Require a verified license if auth is enabled.
	if args.auth_enabled {
		match license_verified {
			#[cfg(debug_assertions)]
			None => {}
			#[cfg(not(debug_assertions))]
			None => return Err(err!("a license is required to enable authentication")),
			Some(false) => return Err(err!("failed to verify license")),
			Some(true) => {}
		}
	}
	let database_url = match args.database_url {
		Some(database_url) => database_url,
		None => default_database_url()?,
	};
	tangram_app::run(tangram_app::Options {
		auth_enabled: args.auth_enabled,
		cookie_domain: args.cookie_domain,
		database_url,
		database_max_connections: args.database_max_connections,
		host: args.host,
		port: args.port,
		sendgrid_api_token: args.sendgrid_api_token,
		stripe_publishable_key: args.stripe_publishable_key,
		stripe_secret_key: args.stripe_secret_key,
		url: args.url,
	})
}

/// Retrieve the user data directory using the `dirs` crate.
#[cfg(feature = "app")]
fn data_dir() -> Result<PathBuf> {
	let data_dir = dirs::data_dir().ok_or_else(|| err!("failed to find user data directory"))?;
	let tangram_data_dir = data_dir.join("tangram");
	std::fs::create_dir_all(&tangram_data_dir).map_err(|_| {
		err!(
			"failed to create tangram data directory in {}",
			tangram_data_dir.display()
		)
	})?;
	Ok(tangram_data_dir)
}

/// Retrieve the default database url, which is a sqlite database in the user data directory.
#[cfg(feature = "app")]
fn default_database_url() -> Result<Url> {
	let tangram_database_path = data_dir()?.join("tangram.db");
	let url = format!(
		"sqlite:{}",
		tangram_database_path.to_str().unwrap().to_owned()
	);
	let url = Url::parse(&url)?;
	Ok(url)
}
