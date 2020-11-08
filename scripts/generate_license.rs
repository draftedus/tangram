use clap::Clap;
use serde_json::json;
use sha2::Digest;
use std::path::PathBuf;
use tangram_util::id::Id;

#[derive(Clap)]
struct Options {
	#[clap(long, about = "the path to the tangram license private key file")]
	private_key: PathBuf,
	#[clap(long, about = "the path to write the license file")]
	output: PathBuf,
}

fn main() {
	let options = Options::parse();
	let tangram_license_private_key = std::fs::read_to_string(options.private_key).unwrap();
	let tangram_license_private_key = tangram_license_private_key
		.lines()
		.skip(1)
		.filter(|line| !line.starts_with('-'))
		.fold(String::new(), |mut data, line| {
			data.push_str(&line);
			data
		});
	let tangram_license_private_key = base64::decode(tangram_license_private_key).unwrap();
	let tangram_license_private_key =
		rsa::RSAPrivateKey::from_pkcs1(&tangram_license_private_key).unwrap();
	let license_data = json!({ "id": Id::new() });
	let license_data = serde_json::to_vec(&license_data).unwrap();
	let mut digest = sha2::Sha256::new();
	digest.update(&license_data);
	let digest = digest.finalize();
	let signature = tangram_license_private_key
		.sign(rsa::PaddingScheme::new_pkcs1v15_sign(None), &digest)
		.unwrap();
	let license_data = base64::encode(license_data);
	let signature = base64::encode(signature);
	let mut license = String::new();
	license.push_str(&license_data);
	license.push(':');
	license.push_str(&signature);
	std::fs::write(options.output, license).unwrap();
}
