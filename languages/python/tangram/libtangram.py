from cffi import FFI
import sys
import os.path
import platform

ffi = FFI()
ffi.cdef("""
	typedef void tangram_model;
	int tangram_model_load(
		const void* model_data,
		size_t model_data_len,
		tangram_model** model
	);
	int tangram_model_predict(
		tangram_model* model,
		const char* input,
		const char* options,
		char** output
	);
	int tangram_model_id(
		tangram_model* model,
		char** output
	);
	int tangram_string_free(const char* ptr);
	int tangram_model_free(tangram_model* model);
""")

machine = platform.machine()
platform = sys.platform
if machine == 'x86_64' and (platform == "linux" or platform == "linux2"):
	library_path = "libtangram/linux_amd64/libtangram.so"
elif machine == 'x86_64' and platform == "darwin":
	library_path = "libtangram/macos_amd64/libtangram.dylib"
elif machine == 'x86_64' and platform == "win32":
	library_path = "libtangram/windows_amd64/tangram.dll"
else:
	raise Exception('tangram-python does not yet support your combination of operating system and CPU architecture. Want support for your platform? Get in touch at help@tangramhq.com.')

libtangram_path = os.path.join(os.path.abspath(os.path.dirname(__file__)), library_path)
libtangram = ffi.dlopen(libtangram_path)
