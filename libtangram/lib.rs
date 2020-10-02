/*!
This crate implements the C API for libtangram, the Tangram C library, which is used by the libraries for each programming language to make predictions using a model trained with the Tangram CLI. While the APIs for the language libraries are stable, this API is unstable and subject to change. All functions return a status code which will be zero on success or non-zero on error.
*/

#![allow(clippy::missing_safety_doc)]

use std::{
	alloc::{alloc, dealloc, Layout},
	convert::TryInto,
	ffi::{CStr, CString},
	panic::catch_unwind,
};
use tangram_core::{predict::PredictInput, predict::PredictModel};

/// Retrieve the version of libtangram that is in use. On success, a pointer to the C string with the version will be written to `version_ptr`. You must call `tangram_string_free` when you are done with it.
#[no_mangle]
pub extern "C" fn tangram_version(version_ptr: *mut *const u8) -> isize {
	let result = catch_unwind(|| unsafe {
		let version = env!("CARGO_PKG_VERSION");
		let version = CString::new(version).unwrap();
		*version_ptr = CString::into_raw(version) as *const u8;
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

/// Load a model from the bytes pointed to by `model_data` with length `model_data_len`. On success, a pointer to the loaded model will be written to `model_ptr`. You must call `tangram_model_free` when you are done with it.
#[no_mangle]
pub extern "C" fn tangram_model_load(
	model_data: *const u8,
	model_data_len: usize,
	model_ptr: *mut *const PredictModel,
) -> isize {
	let result = catch_unwind(|| unsafe {
		assert!(!model_ptr.is_null());
		let bytes = std::slice::from_raw_parts(model_data, model_data_len);
		let model = tangram_core::model::Model::from_slice(bytes).unwrap();
		let model: PredictModel = model.try_into().unwrap();
		let model = Box::new(model);
		*model_ptr = Box::into_raw(model);
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

/// Retrieve the id of the model. On success, a pointer to the model id as a C string will be written to `id_ptr`. You must call `tangram_string_free` when you are done with it.
#[no_mangle]
pub extern "C" fn tangram_model_id(model: *const PredictModel, id_ptr: *mut *const u8) -> isize {
	let result = catch_unwind(|| unsafe {
		let model = model.as_ref().unwrap();
		let id = match model {
			PredictModel::LinearRegressor(model) => &model.id,
			PredictModel::TreeRegressor(model) => &model.id,
			PredictModel::LinearBinaryClassifier(model) => &model.id,
			PredictModel::TreeBinaryClassifier(model) => &model.id,
			PredictModel::LinearMulticlassClassifier(model) => &model.id,
			PredictModel::TreeMulticlassClassifier(model) => &model.id,
		};
		let id = CString::new(id.to_owned()).unwrap();
		*id_ptr = CString::into_raw(id) as *const u8;
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

/// Make a prediction! `model` should point to a model loaded with `tangram_model_load`. `input_ptr` should be a C string of a json serialized PredictInput. On success, a pointer to the output as a json serialized C string will be written to `output_ptr`. You must call `tangram_string_free` when you are done with it.
#[no_mangle]
pub extern "C" fn tangram_model_predict(
	model: *const PredictModel,
	input: *const u8,
	options: *const u8,
	output_ptr: *mut *const u8,
) -> isize {
	let result = catch_unwind(|| unsafe {
		let model = model.as_ref().unwrap();
		let input = CStr::from_ptr(input as *const i8).to_str().unwrap();
		let input: PredictInput = serde_json::from_str(input).unwrap();
		let options: Option<tangram_core::predict::PredictOptions> =
			options.as_ref().map(|options_ptr| {
				let options = CStr::from_ptr(options_ptr as *const u8 as *const i8)
					.to_str()
					.unwrap();
				serde_json::from_str(options).unwrap()
			});
		let output = tangram_core::predict::predict(model, input, options);
		let output = serde_json::to_string(&output).unwrap();
		let output = CString::new(output).unwrap();
		*output_ptr = CString::into_raw(output) as *const u8;
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

/// Free the C string created by libtangram pointed to by `string`.
#[no_mangle]
pub extern "C" fn tangram_string_free(string: *mut u8) -> isize {
	let result = catch_unwind(|| unsafe {
		drop(CString::from_raw(string as *mut i8));
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

/// Free the model pointed to by `model`.
#[no_mangle]
pub extern "C" fn tangram_model_free(model: *mut PredictModel) -> isize {
	let result = catch_unwind(|| unsafe {
		drop(Box::from_raw(model));
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

/// This function exposes the allocator used by libtangram. It is used by the wasm build of libtangram because WebAssembly does not yet include its own allocator.
#[no_mangle]
pub extern "C" fn tangram_alloc(size: usize, align: usize) -> *mut u8 {
	let layout = Layout::from_size_align(size, align).unwrap();
	unsafe { alloc(layout) }
}

/// This function exposes the allocator used by libtangram. It is used by the wasm build of libtangram because WebAssembly does not yet include its own allocator.
#[no_mangle]
pub unsafe extern "C" fn tangram_dealloc(ptr: *mut u8, size: usize, align: usize) {
	if size == 0 {
		return;
	}
	let layout = Layout::from_size_align_unchecked(size, align);
	dealloc(ptr, layout);
}
