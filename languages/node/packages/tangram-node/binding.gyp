{
	"targets": [
		{
			"target_name": "native",
			"sources": ["native.c"],
			"conditions": [
				[
					"OS=='linux'", {
						"libraries": [
							"<(module_root_dir)/libtangram/linux_amd64/libtangram.a"
						],
					}
				],
				[
					"OS=='mac'", {
						"libraries": [
							"<(module_root_dir)/libtangram/macos_amd64/libtangram.a"
						],
					}
				],
				[
					"OS=='win'", {
						"libraries": [
							"<(module_root_dir)/libtangram/windows_amd64/tangram.lib",
							"ws2_32",
							"userenv"
						],
					}
				],
			]
		}
	]
}
