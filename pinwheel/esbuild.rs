use anyhow::{format_err, Result};
use glob::glob;
use std::borrow::Cow;
use std::path::Path;

pub fn esbuild_single_page(root_dir: &Path, out_dir: &Path, page_entry: &str) -> Result<()> {
	esbuild_pages(root_dir, out_dir, &[page_entry.into()])
}

pub fn esbuild_all_pages(root_dir: &Path, out_dir: &Path) -> Result<()> {
	let mut page_entries = Vec::new();
	let pattern = root_dir.join("pages/**/page.tsx");
	let pattern = pattern.to_str().unwrap();
	for entry in glob(pattern)? {
		let entry = entry?
			.strip_prefix(root_dir)
			.unwrap()
			.strip_prefix("pages/")
			.unwrap()
			.parent()
			.unwrap()
			.to_owned();
		let entry = entry.to_str().unwrap().to_owned().into();
		page_entries.push(entry)
	}
	esbuild_pages(root_dir, out_dir, &page_entries)?;
	let static_dir = root_dir.join("static");
	for path in walkdir::WalkDir::new(&static_dir) {
		let path = path?;
		let path = path.path();
		if path.is_file() {
			let out_path = out_dir.join(path.strip_prefix(&static_dir).unwrap());
			std::fs::create_dir_all(out_path.parent().unwrap())?;
			std::fs::copy(path, out_path)?;
		}
	}
	Ok(())
}

pub fn esbuild_pages(root_dir: &Path, out_dir: &Path, page_entries: &[Cow<str>]) -> Result<()> {
	// remove the out_dir if it exists and create it
	if out_dir.exists() {
		std::fs::remove_dir_all(&out_dir)?;
	}
	std::fs::create_dir_all(&out_dir)?;
	let manifest_path = out_dir.join("manifest.json");
	let pinwheel_source_path = root_dir.join("pinwheel.ts");
	let document_source_path = root_dir.join("document.tsx");
	let mut args = vec![
		"--format=esm".to_string(),
		"--minify".to_string(),
		"--bundle".to_string(),
		"--splitting".to_string(),
		"--resolve-extensions=.ts,.tsx,.svg,.png".to_string(),
		"--loader:.svg=dataurl".to_string(),
		"--loader:.png=file".to_string(),
		"--sourcemap".to_string(),
		format!("--metafile={}", manifest_path.display()),
		format!("--outdir={}", out_dir.display()),
		format!("{}", pinwheel_source_path.display()),
		format!("{}", document_source_path.display()),
	];
	for page_entry in page_entries {
		let page_source_path = root_dir
			.join("pages")
			.join(page_entry.as_ref())
			.join("page.tsx");
		let client_js_path = root_dir
			.join("pages")
			.join(page_entry.as_ref())
			.join("client.tsx");
		let client_js_path = if client_js_path.exists() {
			Some(client_js_path)
		} else {
			None
		};
		args.push(format!("{}", page_source_path.display()));
		if let Some(client_js_path) = client_js_path {
			args.push(format!("{}", client_js_path.display()));
		}
	}
	let mut process = std::process::Command::new("esbuild").args(&args).spawn()?;
	let status = process.wait()?;
	if !status.success() {
		return Err(format_err!("esbuild {}", status.to_string()));
	}
	Ok(())
}
