use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};
use std::path::PathBuf;
use tokio::fs;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
	let _version = std::env::var("VERSION").unwrap_or_else(|_| {
		rustyline::Editor::<()>::new()
			.readline("Version: ")
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

	let github_client = GitHubClient::new(username, token);

	// Get the most recent workflow run.
	#[derive(Debug, Deserialize)]
	pub struct WorkflowRunsResponse {
		pub total_count: usize,
		pub workflow_runs: Vec<WorkflowRun>,
	}
	#[derive(Debug, Deserialize)]
	pub struct WorkflowRun {
		pub id: String,
	}
	let workflow_runs_response = github_client
		.request::<WorkflowRunsResponse>(
			reqwest::Method::GET,
			"/repos/tangram-hq/tangram/actions/workflows/build.yaml/runs",
		)
		.await?;
	let workflow_run = workflow_runs_response.workflow_runs.get(0).unwrap();

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
			reqwest::Method::GET,
			&format!(
				"/repos/tangram-hq/tangram/actions/runs/${}/artifacts",
				workflow_run.id
			),
		)
		.await?;
	let artifacts = artifacts_response.artifacts;

	// Download the artifacts.
	for artifact in artifacts {
		let data = reqwest::get(&artifact.archive_download_url)
			.await?
			.bytes()
			.await?;
		let output_path = PathBuf::from("dist").join(artifact.name);
		fs::write(output_path, data).await?;
	}

	Ok(())
}

struct GitHubClient {
	base_url: Url,
	client: reqwest::Client,
	username: String,
	token: String,
}

impl GitHubClient {
	pub fn new(username: String, token: String) -> GitHubClient {
		let base_url = Url::parse("https://api.github.com").unwrap();
		let mut headers = reqwest::header::HeaderMap::new();
		headers.insert(
			reqwest::header::USER_AGENT,
			reqwest::header::HeaderValue::from_str("tangram-hq/tangram build_download script")
				.unwrap(),
		);
		headers.insert(
			reqwest::header::AUTHORIZATION,
			reqwest::header::HeaderValue::from_str(&base64::encode(&format!(
				"{}:{}",
				username, token
			)))
			.unwrap(),
		);
		let client = reqwest::Client::builder()
			.default_headers(headers)
			.build()
			.unwrap();
		GitHubClient {
			base_url,
			client,
			username,
			token,
		}
	}

	async fn request<T>(&self, method: reqwest::Method, path: &str) -> Result<T>
	where
		T: DeserializeOwned,
	{
		let mut url = self.base_url.clone();
		url.set_path(path);
		let value = self
			.client
			.request(method, url)
			.basic_auth(&self.username, Some(&self.token))
			.send()
			.await?
			.json()
			.await?;
		Ok(value)
	}
}

// async function sh(cmd: string) {
// 	await Deno.run({
// 		cmd: ['sh', '-c', cmd],
// 	}).status()
// }

// // Create the linux x64 cli archive.
// await sh(`unzip dist/tangram-cli-linux-x64 -d dist`)
// await sh(`chmod +x dist/tangram`)
// await sh(`tar czf dist/tangram-cli-${version}-linux-x64.tar.gz -C dist tangram`)
// await Deno.remove('dist/tangram')

// // Create the macos x64 cli archive.
// await sh(`unzip dist/tangram-cli-macos-x64 -d dist`)
// await sh(`chmod +x dist/tangram`)
// await sh(`tar czf dist/tangram-cli-${version}-macos-x64.tar.gz -C dist tangram`)
// await Deno.remove('dist/tangram')

// // Create the windows x64 cli archive.
// await sh(`unzip dist/tangram-cli-windows-x64 -d dist`)
// await sh(`zip -qj dist/tangram-cli-${version}-windows-x64.zip dist/tangram.exe`)
// await Deno.remove('dist/tangram.exe')

// // Create the linux x64 dynamic libtangram archive.
// await sh(`unzip dist/tangram-cdylib-linux-x64 -d dist`)
// await sh(
// 	`zip -qj dist/tangram-cdylib-${version}-linux-x64.zip dist/libtangram.so`,
// )
// await Deno.copyFile(
// 	'dist/libtangram.so',
// 	'languages/python/tangram/libtangram-linux-x64.so',
// )
// await Deno.copyFile(
// 	'dist/libtangram.so',
// 	'languages/ruby/lib/tangram/libtangram-linux-x64.so',
// )
// await Deno.remove('dist/libtangram.so')

// // Create the linux x64 static libtangram archive.
// await sh(`unzip dist/tangram-staticlib-linux-x64 -d dist`)
// await sh(
// 	`zip -qj dist/tangram-staticlib-${version}-linux-x64.zip dist/libtangram.a`,
// )
// await Deno.copyFile('dist/libtangram.a', 'languages/go/libtangram-linux-x64.a')
// await Deno.copyFile(
// 	'dist/libtangram.a',
// 	'languages/node/packages/tangram-node/libtangram-linux-x64.a',
// )
// await Deno.copyFile(
// 	'dist/libtangram.a',
// 	'languages/rust/libtangram-linux-x64.a',
// )
// await Deno.remove('dist/libtangram.a')

// // Create the macos x64 dynamic libtangram archive.
// await sh(`unzip dist/tangram-cdylib-macos-x64 -d dist`)
// await sh(
// 	`zip -qj dist/tangram-cdylib-${version}-macos-x64.zip dist/libtangram.dylib`,
// )
// await Deno.copyFile(
// 	'dist/libtangram.dylib',
// 	'languages/python/tangram/libtangram-macos-x64.dylib',
// )
// await Deno.copyFile(
// 	'dist/libtangram.dylib',
// 	'languages/ruby/lib/tangram/libtangram-macos-x64.dylib',
// )
// await Deno.remove('dist/libtangram.dylib')

// // Create the macos x64 static libtangram archive.
// await sh(`unzip dist/tangram-staticlib-macos-x64 -d dist`)
// await sh(
// 	`zip -qj dist/tangram-staticlib-${version}-macos-x64.zip dist/libtangram.a`,
// )
// await Deno.copyFile('dist/libtangram.a', 'languages/go/libtangram-macos-x64.a')
// await Deno.copyFile(
// 	'dist/libtangram.a',
// 	'languages/node/packages/tangram-node/libtangram-macos-x64.a',
// )
// await Deno.copyFile(
// 	'dist/libtangram.a',
// 	'languages/rust/libtangram-macos-x64.a',
// )
// await Deno.remove('dist/libtangram.a')

// // Create the windows x64 dynamic libtangram archive.
// await sh(`unzip dist/tangram-cdylib-windows-x64 -d dist`)
// await sh(
// 	`zip -qj dist/tangram-cdylib-${version}-windows-x64.zip dist/tangram.dll`,
// )
// await Deno.copyFile(
// 	'dist/tangram.dll',
// 	'languages/python/tangram/tangram-windows-x64.dll',
// )
// await Deno.copyFile(
// 	'dist/tangram.dll',
// 	'languages/ruby/lib/tangram/tangram-windows-x64.dll',
// )
// await Deno.remove('dist/tangram.dll')

// // Create the windows x64 static libtangram archive.
// await sh(`unzip dist/tangram-staticlib-windows-x64 -d dist`)
// await sh(
// 	`zip -qj dist/tangram-staticlib-${version}-windows-x64.zip dist/tangram.lib`,
// )
// await Deno.copyFile('dist/tangram.lib', 'languages/go/tangram-windows-x64.lib')
// await Deno.copyFile(
// 	'dist/tangram.lib',
// 	'languages/node/packages/tangram-node/tangram-windows-x64.lib',
// )
// await Deno.copyFile(
// 	'dist/tangram.lib',
// 	'languages/rust/tangram-windows-x64.lib',
// )
// await Deno.remove('dist/tangram.lib')

// // Create the wasm libtangram archive.
// await sh(`unzip dist/tangram-wasm -d dist`)
// await sh(`zip -qj dist/tangram-wasm-${version}.zip dist/tangram.wasm`)
// await Deno.copyFile(
// 	'dist/tangram.wasm',
// 	'languages/js/packages/tangram-js/tangram.wasm',
// )
// await Deno.remove('dist/tangram.wasm')

// // Delete all the downloaded artifacts.
// await Promise.all(
// 	artifacts.map(async (artifact: any) => {
// 		await Deno.remove(join('dist', artifact.name))
// 	}),
// )
