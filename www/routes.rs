use std::collections::BTreeMap;
use tangram_deps::maplit::btreemap;

pub type RouteMap = BTreeMap<&'static str, Box<dyn Fn() -> String + Send + Sync>>;

pub fn build_route_map() -> RouteMap {
	btreemap! {
		"/" => Box::new(tangram_www_pages_index::render) as Box<dyn Fn() -> String + Send + Sync>,
	}
}
