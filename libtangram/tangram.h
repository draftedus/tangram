// After training a model using the tangram command line tool, you can load the `.tangram` file using libtangram to make predictions.
// All functions return a status code which will be zero for success or non-zero to indicate an error has occurred.

// `tangram_model` is an opaque handle to a tangram model.
typedef void tangram_model;

// `tangram_model_load` will read a model from the buffer `model_data` with length `model_data_len`
// and write a handle to it to the pointer `model`.
int tangram_model_load(
	const void* model_data,
	size_t model_data_len,
	tangram_model** model
);

// Use `tangram_model_predict` to make predictions with your model.
// `input` should be a json array of objects, where the keys in each object correspond to the column names in your model.
// `options` should be a json object. See `predict.rs` for the schema.
// `output` will be filled with a json array of objects, equal in length to `input`, where each object contains the prediction information. See `predict.rs` for the schema.
int tangram_model_predict(
	tangram_model* model,
	const char* input,
	const char* options,
	char** output
);

// Get the id of the model. This is used by the tangram language libraries to implement production logging.
int tangram_model_id(
	tangram_model* model,
	char** output
);

// When you are done with a string created by any function in libtangram, call `tangram_string_free` to free it.
int tangram_string_free(const char* ptr);

// When you are done with a model, call `tangram_model_free` to free it.
int tangram_model_free(tangram_model* model);
