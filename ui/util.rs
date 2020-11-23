pub fn format_percent(value: f32) -> String {
	if value.partial_cmp(&1.0) == Some(std::cmp::Ordering::Equal) {
		"100%".to_owned()
	} else {
		let v = value * 100.0;
		format!("{:.2}%", v)
	}
}
