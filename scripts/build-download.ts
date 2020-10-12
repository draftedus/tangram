import { ensureFile } from 'https://deno.land/std@0.74.0/fs/mod.ts'
import { join } from 'https://deno.land/std@0.74.0/path/mod.ts'

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
		'/repos/tangram-hq/tangram/actions/workflows/build.yml/runs',
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

async function sh(cmd: string) {
	await Deno.run({
		cmd: ['sh', '-c', cmd],
	}).status()
}

// Create the linux x64 cli archive.
await sh(`unzip dist/tangram-cli-linux-x64 -d dist`)
await sh(`chmod +x dist/tangram`)
await sh(`tar czf dist/tangram-cli-${version}-linux-x64.tar.gz -C dist tangram`)
await Deno.remove('dist/tangram')

// Create the macos x64 cli archive.
await sh(`unzip dist/tangram-cli-macos-x64 -d dist`)
await sh(`chmod +x dist/tangram`)
await sh(`tar czf dist/tangram-cli-${version}-macos-x64.tar.gz -C dist tangram`)
await Deno.remove('dist/tangram')

// Create the windows x64 cli archive.
await sh(`unzip dist/tangram-cli-windows-x64 -d dist`)
await sh(`zip -qj dist/tangram-cli-${version}-windows-x64.zip dist/tangram.exe`)
await Deno.remove('dist/tangram.exe')

// Create the linux x64 dynamic libtangram archive.
await sh(`unzip dist/tangram-cdylib-linux-x64 -d dist`)
await sh(
	`zip -qj dist/tangram-cdylib-${version}-linux-x64.zip dist/libtangram.so`,
)
await Deno.copyFile(
	'dist/libtangram.so',
	'languages/python/tangram/libtangram-linux-x64.so',
)
await Deno.copyFile(
	'dist/libtangram.so',
	'languages/ruby/lib/tangram/libtangram-linux-x64.so',
)
await Deno.remove('dist/libtangram.so')

// Create the linux x64 static libtangram archive.
await sh(`unzip dist/tangram-staticlib-linux-x64 -d dist`)
await sh(
	`zip -qj dist/tangram-staticlib-${version}-linux-x64.zip dist/libtangram.a`,
)
await Deno.copyFile('dist/libtangram.a', 'languages/go/libtangram-linux-x64.a')
await Deno.copyFile(
	'dist/libtangram.a',
	'languages/node/packages/tangram-node/libtangram-linux-x64.a',
)
await Deno.copyFile(
	'dist/libtangram.a',
	'languages/rust/libtangram-linux-x64.a',
)
await Deno.remove('dist/libtangram.a')

// Create the macos x64 dynamic libtangram archive.
await sh(`unzip dist/tangram-cdylib-macos-x64 -d dist`)
await sh(
	`zip -qj dist/tangram-cdylib-${version}-macos-x64.zip dist/libtangram.dylib`,
)
await Deno.copyFile(
	'dist/libtangram.dylib',
	'languages/python/tangram/libtangram-macos-x64.dylib',
)
await Deno.copyFile(
	'dist/libtangram.dylib',
	'languages/ruby/lib/tangram/libtangram-macos-x64.dylib',
)
await Deno.remove('dist/libtangram.dylib')

// Create the macos x64 static libtangram archive.
await sh(`unzip dist/tangram-staticlib-macos-x64 -d dist`)
await sh(
	`zip -qj dist/tangram-staticlib-${version}-macos-x64.zip dist/libtangram.a`,
)
await Deno.copyFile('dist/libtangram.a', 'languages/go/libtangram-macos-x64.a')
await Deno.copyFile(
	'dist/libtangram.a',
	'languages/node/packages/tangram-node/libtangram-macos-x64.a',
)
await Deno.copyFile(
	'dist/libtangram.a',
	'languages/rust/libtangram-macos-x64.a',
)
await Deno.remove('dist/libtangram.a')

// Create the windows x64 dynamic libtangram archive.
await sh(`unzip dist/tangram-cdylib-windows-x64 -d dist`)
await sh(
	`zip -qj dist/tangram-cdylib-${version}-windows-x64.zip dist/tangram.dll`,
)
await Deno.copyFile(
	'dist/tangram.dll',
	'languages/python/tangram/tangram-windows-x64.dll',
)
await Deno.copyFile(
	'dist/tangram.dll',
	'languages/ruby/lib/tangram/tangram-windows-x64.dll',
)
await Deno.remove('dist/tangram.dll')

// Create the windows x64 static libtangram archive.
await sh(`unzip dist/tangram-staticlib-windows-x64 -d dist`)
await sh(
	`zip -qj dist/tangram-staticlib-${version}-windows-x64.zip dist/tangram.lib`,
)
await Deno.copyFile('dist/tangram.lib', 'languages/go/tangram-windows-x64.lib')
await Deno.copyFile(
	'dist/tangram.lib',
	'languages/node/packages/tangram-node/tangram-windows-x64.lib',
)
await Deno.copyFile(
	'dist/tangram.lib',
	'languages/rust/tangram-windows-x64.lib',
)
await Deno.remove('dist/tangram.lib')

// Create the wasm libtangram archive.
await sh(`unzip dist/tangram-wasm -d dist`)
await sh(`zip -qj dist/tangram-wasm-${version}.zip dist/tangram.wasm`)
await Deno.copyFile(
	'dist/tangram.wasm',
	'languages/js/packages/tangram-js/tangram.wasm',
)
await Deno.remove('dist/tangram.wasm')

// Delete all the downloaded artifacts.
await Promise.all(
	artifacts.map(async (artifact: any) => {
		await Deno.remove(join('dist', artifact.name))
	}),
)
