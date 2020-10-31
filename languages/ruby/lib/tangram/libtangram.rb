require 'ffi'
require 'rbconfig'

module LibTangram
	cpu = RbConfig::CONFIG['host_cpu']
	os = RbConfig::CONFIG['host_os']
	if cpu == 'x86_64' and os =~ /linux/
		library_path = 'libtangram/linux_amd64/libtangram.so'
	elsif cpu == 'x86_64' and os =~ /darwin/
		library_path = 'libtangram/macos_amd64/libtangram.dylib'
	elsif cpu == 'x86_64' and os =~ /mingw/
		library_path = 'libtangram/windows_amd64/tangram.dll'
	else
		raise 'tangram-ruby does not yet support your combination of operating system and CPU architecture. Want support for your platform? Get in touch at help@tangramhq.com.'
	end
	extend FFI::Library
	ffi_lib File.expand_path("#{library_path}", __dir__)
	attach_function :tangram_model_load, [:pointer, :uint, :pointer], :int
	attach_function :tangram_model_id, [:pointer, :pointer], :int
	attach_function :tangram_model_predict, [:pointer, :pointer, :pointer, :pointer], :int
	attach_function :tangram_string_free, [:pointer], :int
	attach_function :tangram_model_free, [:pointer], :int
end
