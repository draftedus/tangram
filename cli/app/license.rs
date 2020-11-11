use rsa::PublicKey;
use sha2::Digest;
use std::path::Path;
use tangram_util::{err, error::Result};

pub fn verify_license(license_file_path: &Path) -> Result<bool> {
	let tangram_license_public_key: &str = "
-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAq+JphywG8wCe6cX+bx4xKH8xphMhaI5BgYefQHUXwp8xavoor6Fy
B54yZba/pkfTnao+P9BvPT0PlSJ1L9aGzq45lcQCcaT+ZdPC5qUogTrKu4eB2qSj
yTt5pGnPsna+/7yh2sDhC/SHMvTPKt4oHgobWYkH3/039Rj7z5X2WGq69gJzSknX
/lraNlVUqCWi3yCnMP9QOV5Tou5gQi4nxlfEJO3razrif5jHw1NufQ+xpx1GCpN9
WhFBU2R4GFZsxlEXV9g1Os1ZpyVuoOe9BnenuS57TixU9SC8kFUHAyAWRSiuLjoP
xAmGGm4wQ4FlMAt+Bj/K6rvdG3FJUu5ttQIDAQAB
-----END RSA PUBLIC KEY-----
";
	let tangram_license_public_key = tangram_license_public_key
		.lines()
		.skip(1)
		.filter(|line| !line.starts_with('-'))
		.fold(String::new(), |mut data, line| {
			data.push_str(&line);
			data
		});
	let tangram_license_public_key = base64::decode(tangram_license_public_key).unwrap();
	let tangram_license_public_key =
		rsa::RSAPublicKey::from_pkcs1(&tangram_license_public_key).unwrap();
	let license_data = std::fs::read(license_file_path)?;
	let mut sections = license_data.split(|byte| *byte == b':');
	let license_data = sections.next().ok_or_else(|| err!("invalid license"))?;
	let license_data = base64::decode(&license_data)?;
	let signature = sections.next().ok_or_else(|| err!("invalid license"))?;
	let signature = base64::decode(&signature)?;
	let mut digest = sha2::Sha256::new();
	digest.update(&license_data);
	let digest = digest.finalize();
	tangram_license_public_key.verify(
		rsa::PaddingScheme::new_pkcs1v15_sign(None),
		&digest,
		&signature,
	)?;
	Ok(true)
}
