let datasets = [
	{ name: 'boston', target: 'medv' },
	{ name: 'heart-disease', target: 'diagnosis' },
	{ name: 'titanic', target: 'survived' },
	{ name: 'census', target: 'income' },
	{ name: 'iris', target: 'species' },
]

for (let dataset of datasets) {
	await Deno.run({
		cmd: [
			'cargo',
			'run',
			'--release',
			'--',
			'train',
			'--file',
			`data/${dataset.name}.csv`,
			'--target',
			dataset.target,
			'--output',
			`data/${dataset.name}.tangram`,
		],
	}).status()
}
