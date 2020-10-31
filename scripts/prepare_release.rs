use serde::{de::DeserializeOwned, Deserialize};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tangram_util::error::Result;
use tokio::fs;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
	// Get the input we need to run the release.
	let version = std::env::var("VERSION").unwrap_or_else(|_| {
		rustyline::Editor::<()>::new()
			.readline("Version: ")
			.unwrap()
	});
	let run_id = std::env::var("ACTION_RUN_ID").unwrap_or_else(|_| {
		rustyline::Editor::<()>::new()
			.readline("GitHub Action Run ID: ")
			.unwrap()
	});
	let username = std::env::var("GITHUB_USERNAME").unwrap_or_else(|_| {
		rustyline::Editor::<()>::new()
			.readline("GitHub Username: ")
			.unwrap()
	});
	let token = std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
		rustyline::Editor::<()>::new()
			.readline("GitHub Token: ")
			.unwrap()
	});

	// Create the GitHub client.
	let github_client = GitHubClient::new();

	// Get the artifacts from the workflow run.
	#[derive(Debug, Deserialize)]
	pub struct ArtifactsResponse {
		pub total_count: usize,
		pub artifacts: Vec<Artifact>,
	}
	#[derive(Debug, Deserialize)]
	pub struct Artifact {
		pub name: String,
		pub archive_download_url: String,
	}
	let artifacts_response = github_client
		.request::<ArtifactsResponse>(
			&username,
			&token,
			reqwest::Method::GET,
			&format!(
				"/repos/tangram-hq/tangram/actions/runs/{}/artifacts",
				run_id
			),
		)
		.await
		.unwrap();
	let artifacts = artifacts_response.artifacts;

	// Make the dist directory if necessary.
	let dist_path = PathBuf::from("dist");
	fs::create_dir_all(&dist_path).await.unwrap();

	// Download the artifacts.
	for artifact in artifacts.iter() {
		let data = github_client
			.client
			.get(&artifact.archive_download_url)
			.basic_auth(&username, Some(&token))
			.send()
			.await
			.unwrap()
			.bytes()
			.await
			.unwrap();
		let artifact_path = dist_path.join(&artifact.name);
		fs::write(artifact_path, data).await.unwrap();
	}

	// Create the cli linux amd64 archive.
	sh("unzip dist/tangram-cli-linux_amd64 -d dist");
	sh("chmod +x dist/tangram");
	sh(format!(
		"tar czf dist/tangram-cli-linux_amd64-{}.tar.gz -C dist tangram",
		version
	));
	fs::remove_file("dist/tangram").await.unwrap();

	// Create the cli macos amd64 archive.
	sh("unzip dist/tangram_cli_macos_amd64 -d dist");
	sh("chmod +x dist/tangram");
	sh(format!(
		"tar czf dist/tangram_cli_macos_amd64-{}.tar.gz -C dist tangram",
		version
	));
	fs::remove_file("dist/tangram").await.unwrap();

	// Create the windows amd64 cli archive.
	sh("unzip dist/tangram_cli_windows_amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram_cli_windows_amd64-{}.zip dist/tangram.exe",
		version
	));
	fs::remove_file("dist/tangram.exe").await.unwrap();

	// Create the dynamic library linux amd64 archive.
	sh("unzip dist/tangram_dynamiclib_linux_amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram_dynamiclib_linux_amd64-{}.zip dist/libtangram.so",
		version
	));
	copy(
		"dist/libtangram.so",
		"languages/python/tangram/libtangram/linux_amd64/libtangram.so",
	)
	.await;
	copy(
		"dist/libtangram.so",
		"languages/ruby/lib/tangram/libtangram/linux_amd64/libtangram.so",
	)
	.await;
	fs::remove_file("dist/libtangram.so").await.unwrap();

	// Create the dynamic library macos amd64 archive.
	sh("unzip dist/tangram-dynamiclib_macos_amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram_dynamiclib_macos_amd64_{}.zip dist/libtangram.dylib",
		version
	));
	copy(
		"dist/libtangram.dylib",
		"languages/python/tangram/libtangram/macos_amd64/libtangram.dylib",
	)
	.await;
	copy(
		"dist/libtangram.dylib",
		"languages/ruby/lib/tangram/libtangram/macos_amd64/libtangram.dylib",
	)
	.await;
	fs::remove_file("dist/libtangram.dylib").await.unwrap();

	// Create the dynamic library windows amd64 archive.
	sh("unzip dist/tangram_dynamiclib_windows_amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram_dynamiclib_windows_amd64-{}.zip dist/tangram.dll",
		version
	));
	copy(
		"dist/tangram.dll",
		"languages/python/tangram/libtangram/windows_amd64/tangram.dll",
	)
	.await;
	copy(
		"dist/tangram.dll",
		"languages/ruby/lib/tangram/libtangram/windows_amd64/tangram.dll",
	)
	.await;
	fs::remove_file("dist/tangram.dll").await.unwrap();

	// Create the static library linux amd64 archive.
	sh("unzip dist/tangram_staticlib_linux_amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram_staticlib_linux_amd64-{}.zip dist/libtangram.a",
		version
	));
	copy(
		"dist/libtangram.a",
		"languages/go/libtangram/linux_amd64/libtangram.a",
	)
	.await;
	copy(
		"dist/libtangram.a",
		"languages/node/packages/tangram-node/libtangram/linux_amd64/libtangram.a",
	)
	.await;
	fs::remove_file("dist/libtangram.a").await.unwrap();

	// Create the static library macos amd64 archive.
	sh("unzip dist/tangram_staticlib_macos_amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram_staticlib_macos_amd64-{}.zip dist/libtangram.a",
		version
	));
	copy(
		"dist/libtangram.a",
		"languages/go/libtangram/macos_amd64/libtangram.a",
	)
	.await;
	copy(
		"dist/libtangram.a",
		"languages/node/packages/tangram-node/libtangram/macos_amd64/libtangram.a",
	)
	.await;
	fs::remove_file("dist/libtangram.a").await.unwrap();

	// Create the static library windows amd64 archive.
	sh("unzip dist/tangram-staticlib-windows-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-staticlib-windows-amd64-{}.zip dist/tangram.lib",
		version
	));
	copy(
		"dist/tangram.lib",
		"languages/go/libtangram/windows-amd64/tangram.lib",
	)
	.await;
	copy(
		"dist/tangram.lib",
		"languages/node/packages/tangram-node/libtangram/windows-amd64/tangram.lib",
	)
	.await;
	fs::remove_file("dist/tangram.lib").await.unwrap();

	// Delete all the downloaded artifacts.
	for artifact in artifacts.iter() {
		let artifact_path = dist_path.join(&artifact.name);
		fs::remove_file(artifact_path).await.unwrap();
	}

	Ok(())
}

fn sh<T>(cmd: T)
where
	T: AsRef<str>,
{
	let mut command = std::process::Command::new("sh")
		.stdin(std::process::Stdio::piped())
		.spawn()
		.unwrap();
	command
		.stdin
		.as_mut()
		.unwrap()
		.write_all(cmd.as_ref().as_bytes())
		.unwrap();
	let exit_status = command.wait().unwrap();
	if !exit_status.success() {
		panic!();
	}
}

async fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
	fs::create_dir_all(dst.as_ref().parent().unwrap())
		.await
		.unwrap();
	fs::copy(src, dst).await.unwrap();
}

struct GitHubClient {
	base_url: Url,
	client: reqwest::Client,
}

impl GitHubClient {
	pub fn new() -> GitHubClient {
		let base_url = Url::parse("https://api.github.com").unwrap();
		let mut headers = reqwest::header::HeaderMap::new();
		headers.insert(
			reqwest::header::USER_AGENT,
			reqwest::header::HeaderValue::from_str("tangram-hq/tangram release script").unwrap(),
		);
		let client = reqwest::Client::builder()
			.default_headers(headers)
			.build()
			.unwrap();
		GitHubClient { base_url, client }
	}

	async fn request<T>(
		&self,
		username: &str,
		token: &str,
		method: reqwest::Method,
		path: &str,
	) -> Result<T>
	where
		T: DeserializeOwned,
	{
		let mut url = self.base_url.clone();
		url.set_path(path);
		let value = self
			.client
			.request(method, url)
			.basic_auth(&username, Some(&token))
			.send()
			.await?
			.json()
			.await?;
		Ok(value)
	}
}
