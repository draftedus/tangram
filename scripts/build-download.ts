import { ensureFile } from 'https://deno.land/std@0.61.0/fs/mod.ts'
import { join } from 'https://deno.land/std@0.61.0/path/mod.ts'

let version = Deno.env.get('VERSION')
let Authorization = `Basic ${btoa(
	`${Deno.env.get('GITHUB_USERNAME')}:${Deno.env.get('GITHUB_TOKEN')}`,
)}`

async function githubRequest(method: string, path: string, body?: any) {
	let response = await fetch('https://api.github.com' + path, {
		body,
		headers: { Authorization },
		method,
	})
	if (!response.ok) {
		throw Error(response.statusText)
	}
	return await response.json()
}

// Get the most recent workflow run.
let run = (
	await githubRequest(
		'GET',
		`/repos/tangram-hq/tangram/actions/workflows/build.yml/runs`,
	)
).workflow_runs[0]

// Get the artifacts from the workflow run.
let artifacts = (
	await githubRequest(
		'GET',
		`/repos/tangram-hq/tangram/actions/runs/${run.id}/artifacts`,
	)
).artifacts

// Download the artifacts.
await Promise.all(
	artifacts.map(async (artifact: any) => {
		let response = await fetch(artifact.archive_download_url, {
			headers: { Authorization },
		})
		if (!response.ok) {
			throw Error(response.statusText)
		}
		let data = await response.arrayBuffer()
		let outputPath = join('dist', artifact.name)
		await ensureFile(outputPath)
		await Deno.writeFile(outputPath, new Uint8Array(data))
	}),
)

let script = `

	unzip dist/tangram-cli-linux-x64 -d dist
	chmod +x dist/tangram
	tar czf dist/tangram-cli-${version}-linux-x64.tar.gz -C dist tangram
	rm dist/tangram

	unzip dist/tangram-cli-macos-x64 -d dist
	chmod +x dist/tangram
	tar czf dist/tangram-cli-${version}-macos-x64.tar.gz -C dist tangram
	rm dist/tangram

	unzip dist/tangram-cli-windows-x64 -d dist
	zip -qj dist/tangram-cli-${version}-windows-x64.zip dist/tangram.exe
	rm dist/tangram.exe

	unzip dist/tangram-cdylib-linux-x64 -d dist
	zip -qj dist/tangram-cdylib-${version}-linux-x64.zip dist/libtangram.so
	cp dist/libtangram.so languages/python/tangram/libtangram-linux-x64.so
	cp dist/libtangram.so languages/ruby/lib/tangram/libtangram-linux-x64.so
	rm dist/libtangram.so

	unzip dist/tangram-staticlib-linux-x64 -d dist
	zip -qj dist/tangram-staticlib-${version}-linux-x64.zip dist/libtangram.a
	cp dist/libtangram.a languages/go/libtangram-linux-x64.a
	cp dist/libtangram.a languages/node/packages/tangram-node/libtangram-linux-x64.a
	cp dist/libtangram.a languages/rust/libtangram-linux-x64.a
	rm dist/libtangram.a

	unzip dist/tangram-cdylib-macos-x64 -d dist
	zip -qj dist/tangram-cdylib-${version}-macos-x64.zip dist/libtangram.dylib
	cp dist/libtangram.dylib languages/python/tangram/libtangram-macos-x64.dylib
	cp dist/libtangram.dylib languages/ruby/lib/tangram/libtangram-macos-x64.dylib
	rm dist/libtangram.dylib

	unzip dist/tangram-staticlib-macos-x64 -d dist
	zip -qj dist/tangram-staticlib-${version}-macos-x64.zip dist/libtangram.a
	cp dist/libtangram.a languages/go/libtangram-macos-x64.a
	cp dist/libtangram.a languages/node/packages/tangram-node/libtangram-macos-x64.a
	cp dist/libtangram.a languages/rust/libtangram-macos-x64.a
	rm dist/libtangram.a

	unzip dist/tangram-cdylib-windows-x64 -d dist
	zip -qj dist/tangram-cdylib-${version}-windows-x64.zip dist/tangram.dll
	cp dist/tangram.dll languages/python/tangram/tangram-windows-x64.dll
	cp dist/tangram.dll languages/ruby/lib/tangram/tangram-windows-x64.dll
	rm dist/tangram.dll

	unzip dist/tangram-staticlib-windows-x64 -d dist
	zip -qj dist/tangram-staticlib-${version}-windows-x64.zip dist/tangram.lib
	cp dist/tangram.lib languages/go/tangram-windows-x64.lib
	cp dist/tangram.lib languages/node/packages/tangram-node/tangram-windows-x64.lib
	cp dist/tangram.lib languages/rust/tangram-windows-x64.lib
	rm dist/tangram.lib

	unzip dist/tangram-wasm -d dist
	zip -qj dist/tangram-wasm-${version}.zip dist/tangram.wasm
	cp dist/tangram.wasm languages/js/packages/tangram-js/tangram.wasm
	rm dist/tangram.wasm

`

await Deno.run({ cmd: ['sh', '-c', script] }).status()

await Promise.all(
	artifacts.map(async (artifact: any) => {
		await Deno.remove(join('dist', artifact.name))
	}),
)
