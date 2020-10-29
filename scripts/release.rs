use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};
use std::io::Write;
use std::path::PathBuf;
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
	sh("unzip dist/tangram-cli-linux-amd64 -d dist");
	sh("chmod +x dist/tangram");
	sh(format!(
		"tar czf dist/tangram-cli-linux-amd64-{}.tar.gz -C dist tangram",
		version
	));
	fs::remove_file("dist/tangram").await.unwrap();

	// Create the cli macos amd64 archive.
	sh("unzip dist/tangram-cli-macos-amd64 -d dist");
	sh("chmod +x dist/tangram");
	sh(format!(
		"tar czf dist/tangram-cli-macos-amd64-{}.tar.gz -C dist tangram",
		version
	));
	fs::remove_file("dist/tangram").await.unwrap();

	// Create the windows amd64 cli archive.
	sh("unzip dist/tangram-cli-windows-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-cli-windows-amd64-{}.zip dist/tangram.exe",
		version
	));
	fs::remove_file("dist/tangram.exe").await.unwrap();

	// Create the dynamic library linux amd64 archive.
	sh("unzip dist/tangram-dynamiclib-linux-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-dynamiclib-linux-amd64-{}.zip dist/libtangram.so",
		version
	));
	fs::copy(
		"dist/libtangram.so",
		"languages/python/tangram/libtangram-linux-amd64.so",
	)
	.await
	.unwrap();
	fs::copy(
		"dist/libtangram.so",
		"languages/ruby/lib/tangram/libtangram-linux-amd64.so",
	)
	.await
	.unwrap();
	fs::remove_file("dist/libtangram.so").await.unwrap();

	// Create the dynamic library macos amd64 archive.
	sh("unzip dist/tangram-dynamiclib-macos-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-dynamiclib-macos-amd64-{}.zip dist/libtangram.dylib",
		version
	));
	fs::copy(
		"dist/libtangram.dylib",
		"languages/python/tangram/libtangram-macos-amd64.dylib",
	)
	.await
	.unwrap();
	fs::copy(
		"dist/libtangram.dylib",
		"languages/ruby/lib/tangram/libtangram-macos-amd64.dylib",
	)
	.await
	.unwrap();
	fs::remove_file("dist/libtangram.dylib").await.unwrap();

	// Create the dynamic library windows amd64 archive.
	sh("unzip dist/tangram-dynamiclib-windows-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-dynamiclib-windows-amd64-{}.zip dist/tangram.dll",
		version
	));
	fs::copy(
		"dist/tangram.dll",
		"languages/python/tangram/tangram-windows-amd64.dll",
	)
	.await
	.unwrap();
	fs::copy(
		"dist/tangram.dll",
		"languages/ruby/lib/tangram/tangram-windows-amd64.dll",
	)
	.await
	.unwrap();
	fs::remove_file("dist/tangram.dll").await.unwrap();

	// Create the static library linux amd64 archive.
	sh("unzip dist/tangram-staticlib-linux-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-staticlib-linux-amd64-{}.zip dist/libtangram.a",
		version
	));
	fs::copy("dist/libtangram.a", "languages/go/libtangram-linux-amd64.a")
		.await
		.unwrap();
	fs::copy(
		"dist/libtangram.a",
		"languages/node/packages/tangram-node/libtangram-linux-amd64.a",
	)
	.await
	.unwrap();
	fs::remove_file("dist/libtangram.a").await.unwrap();

	// Create the static library macos amd64 archive.
	sh("unzip dist/tangram-staticlib-macos-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-staticlib-macos-amd64-{}.zip dist/libtangram.a",
		version
	));
	fs::copy("dist/libtangram.a", "languages/go/libtangram-macos-amd64.a")
		.await
		.unwrap();
	fs::copy(
		"dist/libtangram.a",
		"languages/node/packages/tangram-node/libtangram-macos-amd64.a",
	)
	.await
	.unwrap();
	fs::remove_file("dist/libtangram.a").await.unwrap();

	// Create the static library windows amd64 archive.
	sh("unzip dist/tangram-staticlib-windows-amd64 -d dist");
	sh(format!(
		"zip -qj dist/tangram-staticlib-windows-amd64-{}.zip dist/tangram.lib",
		version
	));
	fs::copy("dist/tangram.lib", "languages/go/tangram-windows-amd64.lib")
		.await
		.unwrap();
	fs::copy(
		"dist/tangram.lib",
		"languages/node/packages/tangram-node/tangram-windows-amd64.lib",
	)
	.await
	.unwrap();
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
