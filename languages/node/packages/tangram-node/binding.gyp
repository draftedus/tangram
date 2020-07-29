{
  "targets": [
    {
      "target_name": "native",
      "sources": [ "native.c" ],
      "conditions": [
        [
          "OS=='mac'", {
            "libraries": [
              "<(module_root_dir)/libtangram-macos-x64.a"
            ],
          }
        ],
        [
          "OS=='linux'", {
            "libraries": [
              "<(module_root_dir)/libtangram-linux-x64.a"
            ],
          }
        ],
        [
          "OS=='win'", {
            "libraries": [
              "<(module_root_dir)/tangram-windows-x64.lib",
              "ws2_32",
              "userenv"
            ],
          }
        ],
      ]
    }
  ]
}
