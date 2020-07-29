//! This module implements the C api for libtangram, the tangram C library, which is used by the libraries for each programming language to make predictions using a model trained with the the tangram cli.

#![allow(clippy::missing_safety_doc)]

use crate::{predict::PredictInput, predict::PredictModel, types};
use std::{
	alloc::{alloc, dealloc, Layout},
	convert::TryInto,
	ffi::{CStr, CString},
	panic::catch_unwind,
};

#[no_mangle]
pub extern "C" fn tangram_version(version_ptr: *mut *const u8) -> isize {
	let result = catch_unwind(|| unsafe {
		let version = clap::crate_version!();
		let version = CString::new(version).unwrap();
		*version_ptr = CString::into_raw(version) as *const u8;
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

#[no_mangle]
pub extern "C" fn tangram_model_load(
	model_data_ptr: *const u8,
	model_data_len: usize,
	model_ptr: *mut *const PredictModel,
) -> isize {
	let result = catch_unwind(|| unsafe {
		assert!(!model_ptr.is_null());
		let bytes = std::slice::from_raw_parts(model_data_ptr, model_data_len);
		let model = types::Model::from_slice(bytes).unwrap();
		let model: PredictModel = model.try_into().unwrap();
		let model = Box::new(model);
		*model_ptr = Box::into_raw(model);
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

#[no_mangle]
pub extern "C" fn tangram_model_id(model: *const PredictModel, id_ptr: *mut *const u8) -> isize {
	let result = catch_unwind(|| unsafe {
		let model = model.as_ref().unwrap();
		let id = match model {
			PredictModel::LinearRegressor(model) => &model.id,
			PredictModel::GbtRegressor(model) => &model.id,
			PredictModel::LinearBinaryClassifier(model) => &model.id,
			PredictModel::GbtBinaryClassifier(model) => &model.id,
			PredictModel::LinearMulticlassClassifier(model) => &model.id,
			PredictModel::GbtMulticlassClassifier(model) => &model.id,
		};
		let id = CString::new(id.to_owned()).unwrap();
		*id_ptr = CString::into_raw(id) as *const u8;
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

#[no_mangle]
pub extern "C" fn tangram_model_predict(
	model_ptr: *const PredictModel,
	input_ptr: *const u8,
	options_ptr: *const u8,
	output_ptr: *mut *const u8,
) -> isize {
	let result = catch_unwind(|| unsafe {
		let model = model_ptr.as_ref().unwrap();
		let input = CStr::from_ptr(input_ptr as *const i8).to_str().unwrap();
		let input: PredictInput = serde_json::from_str(input).unwrap();
		let options: Option<crate::predict::PredictOptions> =
			options_ptr.as_ref().map(|options_ptr| {
				let options = CStr::from_ptr(options_ptr as *const u8 as *const i8)
					.to_str()
					.unwrap();
				serde_json::from_str(options).unwrap()
			});
		let output = crate::predict::predict(model, input, options);
		let output = serde_json::to_string(&output).unwrap();
		let output = CString::new(output).unwrap();
		*output_ptr = CString::into_raw(output) as *const u8;
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

#[no_mangle]
pub extern "C" fn tangram_string_free(ptr: *mut u8) -> isize {
	let result = catch_unwind(|| unsafe {
		drop(CString::from_raw(ptr as *mut i8));
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

#[no_mangle]
pub extern "C" fn tangram_model_free(model_ptr: *mut PredictModel) -> isize {
	let result = catch_unwind(|| unsafe {
		drop(Box::from_raw(model_ptr));
	});
	match result {
		Ok(_) => 0,
		Err(_) => 1,
	}
}

#[no_mangle]
pub unsafe extern "C" fn tangram_alloc(size: usize) -> *mut u8 {
	let align = std::mem::align_of::<usize>();
	let layout = Layout::from_size_align(size, align).unwrap();
	alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn tangram_dealloc(ptr: *mut u8, size: usize) {
	if size == 0 {
		return;
	}
	let align = std::mem::align_of::<usize>();
	let layout = Layout::from_size_align_unchecked(size, align);
	dealloc(ptr, layout);
}
