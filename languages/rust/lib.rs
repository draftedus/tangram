use libc::{c_char, c_int, c_void, size_t};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;

#[allow(non_camel_case_types)]
type tangram_model = c_void;

extern "C" {
	fn tangram_model_load(
		model_data: *const c_void,
		model_data_len: size_t,
		model: *mut *const tangram_model,
	) -> c_int;
	fn tangram_model_predict(
		model: *const tangram_model,
		input: *const c_char,
		options: *const c_char,
		output: *mut *const c_char,
	) -> c_int;
	fn tangram_model_id(model: *const tangram_model, output: *mut *const c_char) -> c_int;
	fn tangram_string_free(model: *const c_char) -> c_int;
	fn tangram_model_free(model: *const tangram_model) -> c_int;
}

pub struct Model<I, O = Output>
where
	I: serde::Serialize,
	O: OutputTrait,
{
	model_ptr: *const tangram_model,
	input_marker: PhantomData<I>,
	output_marker: PhantomData<O>,
}

#[derive(Debug, serde::Serialize)]
pub struct PredictOptions {
	pub threshold: f32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Output {
	Regression(RegressionOutput),
	Classification(ClassificationOutput),
}

#[derive(Debug, serde::Deserialize)]
pub struct RegressionOutput {
	pub value: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct ClassificationOutput<C = String>
where
	C: std::cmp::Ord,
{
	#[serde(rename = "className")]
	pub class_name: C,
	// pub probabilities: Map<C, f32>,
}

pub trait OutputTrait: serde::de::DeserializeOwned {}

impl OutputTrait for Output {}
impl OutputTrait for RegressionOutput {}
impl<C> OutputTrait for ClassificationOutput<C> where C: std::cmp::Ord + serde::de::DeserializeOwned {}

impl<I, O> Model<I, O>
where
	I: serde::Serialize,
	O: OutputTrait,
{
	pub fn from_slice(model_data: &[u8]) -> Self {
		let mut model_ptr = std::mem::MaybeUninit::uninit();
		let result = unsafe {
			tangram_model_load(
				model_data.as_ptr() as *const c_void,
				model_data.len(),
				model_ptr.as_mut_ptr(),
			)
		};
		assert!(result == 0);
		let model_ptr = unsafe { model_ptr.assume_init() };
		Model {
			model_ptr,
			input_marker: PhantomData,
			output_marker: PhantomData,
		}
	}

	pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
		let bytes = std::fs::read(path).unwrap();
		Self::from_slice(&bytes)
	}

	pub fn predict(&self, input: &[&I], options: Option<&PredictOptions>) -> Vec<O> {
		let mut output_ptr = std::mem::MaybeUninit::uninit();
		let input = serde_json::to_vec(input).unwrap();
		let input = CString::new(input).unwrap();
		let input_ptr = input.as_c_str().as_ptr();
		let options = options.map(|options| {
			let options = serde_json::to_vec(options).unwrap();
			CString::new(options).unwrap()
		});
		let options_ptr = options
			.as_ref()
			.map(|options| options.as_c_str().as_ptr())
			.unwrap_or(std::ptr::null());
		let result = unsafe {
			tangram_model_predict(
				self.model_ptr,
				input_ptr,
				options_ptr,
				output_ptr.as_mut_ptr(),
			)
		};
		assert!(result == 0);
		let output_ptr = unsafe { output_ptr.assume_init() };
		let output = unsafe { std::ffi::CStr::from_ptr(output_ptr) };
		let output = output.to_str().unwrap();
		let output = serde_json::from_str(output).unwrap();
		let result = unsafe { tangram_string_free(output_ptr) };
		assert!(result == 0);
		output
	}

	pub fn id(&self) -> String {
		let mut id_ptr = std::mem::MaybeUninit::uninit();
		let result = unsafe { tangram_model_id(self.model_ptr, id_ptr.as_mut_ptr()) };
		assert!(result == 0);
		let id_ptr = unsafe { id_ptr.assume_init() };
		let id = unsafe { CStr::from_ptr(id_ptr) };
		let id = id.to_str().unwrap().to_owned();
		let result = unsafe { tangram_string_free(id_ptr) };
		assert!(result == 0);
		id
	}
}

impl<I, O> Drop for Model<I, O>
where
	I: serde::Serialize,
	O: OutputTrait,
{
	fn drop(&mut self) {
		let result = unsafe { tangram_model_free(self.model_ptr) };
		assert!(result == 0);
	}
}
