use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use walkdir::WalkDir;

#[proc_macro]
pub fn include_out_dir(_: TokenStream) -> TokenStream {
	let out_dir = std::env::var("OUT_DIR").unwrap();
	let absolute_paths: Vec<PathBuf> = WalkDir::new(&out_dir)
		.into_iter()
		.filter_map(|entry| {
			let entry = entry.unwrap();
			let path = entry.path().to_owned();
			let metadata = std::fs::metadata(&path).unwrap();
			if metadata.is_file() {
				Some(path)
			} else {
				None
			}
		})
		.collect();
	let relative_paths: Vec<PathBuf> = absolute_paths
		.iter()
		.map(|absolute_path| absolute_path.strip_prefix(&out_dir).unwrap().to_owned())
		.collect();
	let absolute_paths: Vec<String> = absolute_paths
		.into_iter()
		.map(|path| path.to_str().unwrap().to_owned())
		.collect();
	let relative_paths: Vec<String> = relative_paths
		.into_iter()
		.map(|path| path.to_str().unwrap().to_owned())
		.collect();
	let ast = quote! {{
		let mut map = ::std::collections::HashMap::new();
		#(
			map.insert(Path::new(#relative_paths), include_bytes!(#absolute_paths).as_ref());
		)*
		include_out_dir::Dir::new(map)
	}};
	ast.into()
}
