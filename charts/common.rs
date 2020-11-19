#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Point {
	x: f32,
	y: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ChartBox {
	h: f32,
	w: f32,
	x: f32,
	y: f32,
}

// The interval is k * 10 ** p. k will always be 1, 2, or 5.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct GridLineInterval {
	k: usize,
	p: usize,
}

pub fn compute_boxes(options: ComputeBoxesOptions) {}
