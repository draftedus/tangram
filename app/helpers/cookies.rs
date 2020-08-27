use std::collections::BTreeMap;

pub fn parse_cookies(cookies_str: &str) -> Result<BTreeMap<&str, &str>, ()> {
	cookies_str
		.split("; ")
		.map(|cookie| {
			let mut components = cookie.split('=');
			let key = match components.next() {
				Some(key) => key,
				None => return Err(()),
			};
			let value = match components.next() {
				Some(value) => value,
				None => return Err(()),
			};
			Ok((key, value))
		})
		.collect()
}
