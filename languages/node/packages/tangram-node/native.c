#include <assert.h>
#include <node_api.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "tangram.h"

napi_value model_load_fn(napi_env env, napi_callback_info info) {
	napi_status status;

	// Get the args.
	size_t argc = 1;
	napi_value argv[1];
	status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
	assert(status == napi_ok);
	if (argc != 1) {
		napi_throw_type_error(env, NULL, "tangram.model_load: incorrect number of arguments");
		return 0;
	}

	// Get the model data.
	napi_value model_data_arg = argv[0];
	bool model_data_arg_is_buffer;
	status = napi_is_buffer(env, model_data_arg, &model_data_arg_is_buffer);
	assert(status == napi_ok);
	if (!model_data_arg_is_buffer) {
		napi_throw_type_error(env, NULL, "tangram.model_load: the first argument must be a buffer");
		return NULL;
	}
	size_t model_data_length;
	void* model_data;
	status = napi_get_buffer_info(env, model_data_arg, &model_data, &model_data_length);
	assert(status == napi_ok);

	// Call `tangram_model_load`.
	tangram_model* model;
	int load_result = tangram_model_load(model_data, model_data_length, &model);
	assert(load_result == 0);

	// Wrap the model in a node external value.
	napi_value model_value;
	void finalize_model(napi_env env, void* finalize_data, void* finalize_hint) {
		tangram_model_free(finalize_data);
	}
	status = napi_create_external(env, model, finalize_model, NULL, &model_value);
	assert(status == napi_ok);

	return model_value;
}

napi_value model_id_fn(napi_env env, napi_callback_info info) {
	napi_status status;

	// Get the args.
	size_t argc = 1;
	napi_value argv[1];
	status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
	assert(status == napi_ok);
	if (argc != 1) {
		napi_throw_type_error(env, NULL, "tangram.model_predict: incorrect number of arguments");
		return NULL;
	}

	// Get the model arg.
	napi_value model_arg = argv[0];
	napi_valuetype model_arg_type;
	status = napi_typeof(env, model_arg, &model_arg_type);
	assert(status == napi_ok);
	if (model_arg_type != napi_external) {
		napi_throw_type_error(env, NULL, "tangram.model_predict: the first argument must be a model");
		return NULL;
	}
	void* model;
	status = napi_get_value_external(env, argv[0], &model);
	assert(status == napi_ok);

	// Call `tangram_model_id`.
	char* id;
	int id_result = tangram_model_id(model, &id);
	assert(id_result == 0);

	// Get the id.
	napi_value id_value;
	status = napi_create_string_utf8(env, id, strlen(id), &id_value);
	assert(status == napi_ok);
	int id_free_result = tangram_string_free(id);
	assert(id_free_result == 0);

	return id_value;
}

napi_value model_predict_fn(napi_env env, napi_callback_info info) {
	napi_status status;

	// Get the args.
	size_t argc = 3;
	napi_value argv[3];
	status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
	assert(status == napi_ok);
	if (argc != 3) {
		napi_throw_type_error(env, NULL, "tangram.model_predict: incorrect number of arguments");
		return NULL;
	}

	// Get the model arg.
	napi_value model_arg = argv[0];
	napi_valuetype model_arg_type;
	status = napi_typeof(env, model_arg, &model_arg_type);
	assert(status == napi_ok);
	if (model_arg_type != napi_external) {
		napi_throw_type_error(env, NULL, "tangram.model_predict: the first argument must be a model");
		return NULL;
	}
	void* model;
	status = napi_get_value_external(env, argv[0], &model);
	assert(status == napi_ok);

	// Get the input arg.
	napi_value input_arg = argv[1];
	napi_valuetype input_arg_type;
	status = napi_typeof(env, argv[1], &input_arg_type);
	assert(status == napi_ok);
	if (input_arg_type != napi_string) {
		napi_throw_type_error(env, NULL, "tangram.model_predict: the second argument must be a string");
		return NULL;
	}
	size_t input_n_bytes;
	status = napi_get_value_string_utf8(env, input_arg, NULL, 0, &input_n_bytes);
	assert(status == napi_ok);
	char* input = (char*)malloc(input_n_bytes + 1);
	status = napi_get_value_string_utf8(env, input_arg, input, input_n_bytes + 1, &input_n_bytes);
	assert(status == napi_ok);

	// Get the options arg.
	napi_value options_arg = argv[2];
	napi_valuetype options_arg_type;
	status = napi_typeof(env, argv[2], &options_arg_type);
	assert(status == napi_ok);
	if (options_arg_type != napi_string && options_arg_type != napi_undefined) {
		napi_throw_type_error(env, NULL, "tangram.model_predict: the third argument must be a string or undefined");
		return NULL;
	}
	char* options = NULL;
	if (options_arg_type == napi_string) {
		size_t options_n_bytes;
		status = napi_get_value_string_utf8(env, options_arg, NULL, 0, &options_n_bytes);
		assert(status == napi_ok);
		options = (char*)malloc(options_n_bytes + 1);
		status = napi_get_value_string_utf8(env, options_arg, options, options_n_bytes + 1, &options_n_bytes);
		assert(status == napi_ok);
	}

	// Call `tangram_model_predict`.
	char* output;
	int predict_result = tangram_model_predict(model, input, options, &output);
	assert(predict_result == 0);

	// Get the output string.
	napi_value output_value;
	status = napi_create_string_utf8(env, output, strlen(output), &output_value);
	assert(status == napi_ok);

	free(input);
	if (options) {
		free(options);
	}
	int output_free_result = tangram_string_free(output);
	assert(output_free_result == 0);

	return output_value;
}

napi_value init(napi_env env, napi_value exports) {
	napi_status status;
	size_t n_prop_descs = 3;
	napi_property_descriptor prop_descs[] = {
		{ "model_load", 0, model_load_fn, 0, 0, 0, napi_default, 0 },
		{ "model_id", 0, model_id_fn, 0, 0, 0, napi_default, 0 },
		{ "model_predict", 0, model_predict_fn, 0, 0, 0, napi_default, 0 }
	};
	status = napi_define_properties(env, exports, n_prop_descs, prop_descs);
	assert(status == napi_ok);
	return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, init);
