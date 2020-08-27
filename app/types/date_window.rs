#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub enum DateWindow {
	#[serde(rename = "today")]
	Today,
	#[serde(rename = "this_month")]
	ThisMonth,
	#[serde(rename = "this_year")]
	ThisYear,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DateWindowInterval {
	Hourly,
	Daily,
	Monthly,
}
