let datasets = [
	{ name: 'boston', target: 'medv' },
	{ name: 'heart-disease', target: 'diagnosis' },
	{ name: 'titanic', target: 'survived' },
	{ name: 'census', target: 'income' },
	{ name: 'iris', target: 'species' },
]

let cmds = []
for (let dataset of datasets) {
	let cmd = `cargo run --release -p tangram -- train --file data/${dataset.name}.csv --target ${dataset.target} --output data/${dataset.name}.tangram`
	cmds.push(cmd)
}

await Deno.run({ cmd: ['sh', '-c', cmds.join('\n')] }).status()
