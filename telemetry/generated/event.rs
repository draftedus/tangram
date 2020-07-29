#![allow(clippy::all)]

use buffy::prelude::*;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum Event {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Run(RunEvent),
	#[buffy(id = 2)]
	Crash(CrashEvent),
	#[buffy(id = 3)]
	Train(TrainEvent),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct RunEvent {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub host_info: buffy::Field<HostInfo>,
	#[buffy(id = 2)]
	pub command: buffy::Field<String>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CrashEvent {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub host_info: buffy::Field<HostInfo>,
	#[buffy(id = 2)]
	pub command: buffy::Field<String>,
	#[buffy(id = 3)]
	pub message: buffy::Field<String>,
	#[buffy(id = 4)]
	pub backtrace: buffy::Field<String>,
	#[buffy(id = 5)]
	pub email: buffy::Field<String>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct TrainEvent {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub host_info: buffy::Field<HostInfo>,
	#[buffy(id = 2)]
	pub command: buffy::Field<String>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct HostInfo {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub operating_system: buffy::Field<OperatingSystem>,
	#[buffy(id = 2)]
	pub cli_version: buffy::Field<String>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum OperatingSystem {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Unknown,
	#[buffy(id = 2)]
	Linux,
	#[buffy(id = 3)]
	MacOs,
	#[buffy(id = 4)]
	Windows,
}
