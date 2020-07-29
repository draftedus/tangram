import * as utf8 from 'https://deno.land/std@0.61.0/encoding/utf8.ts'
import * as csv from 'https://deno.land/std@v0.61.0/encoding/csv.ts'

type TrueValue = {
	date: string
	identifier: string | number
	modelId: string
	trueValue: any
	type: 'true_value'
}

type Prediction = {
	date: string
	identifier: string | number
	input: any
	modelId: string
	output: any
	type: 'prediction'
}

type Config = {
	csvPath: string
	modelId: string
	name: string
	target: string
	targetValues?: string[]
}

type NetworkConfig = {
	url: string
}

let heartDisease: Config = {
	csvPath: './data/heart-disease.csv',
	modelId: 'eb1f5bacbc88a53bada8cbe0d2a5a4a0',
	name: 'heart-disease',
	target: 'diagnosis',
	targetValues: ['Positive', 'Negative'],
}

let boston: Config = {
	csvPath: './data/boston.csv',
	modelId: '1b1ea6e7-70db-4ce0-a816-557242fe35f9',
	name: 'boston',
	target: 'medv',
}

let iris: Config = {
	csvPath: './data/iris.csv',
	modelId: 'b5d3e68b2116d84ef4620b36087aaa74',
	name: 'iris',
	target: 'species',
	targetValues: ['Iris Setosa', 'Iris Virginica', 'Iris Versicolor'],
}

let networkConfig: NetworkConfig = {
	url: 'http://localhost:8001/api/track',
}

let config = heartDisease

let csvData = await Deno.readFile(config.csvPath)
let rows = (await csv.parse(utf8.decode(csvData), {
	header: true,
})) as Array<{ [key: string]: string }>

let nRows = rows.length

async function track(data: Prediction | TrueValue) {
	let response = await fetch(networkConfig.url, {
		body: JSON.stringify(data),
		headers: {
			['content-type']: 'application/json',
		},
		method: 'POST',
	})
	await response.blob()
}

for (let i = 0; i < 10_000; i++) {
	let input = rows[i % nRows]
	let startTime = 1577836800000 // Jan 1 2020 00:00:00
	let endTime = Date.now()
	let time = Math.random() * (endTime - startTime) + startTime
	let date = new Date(time)
	let output
	if (config.name == 'heart-disease') {
		if (!config.targetValues) {
			throw Error()
		}
		if (input['chest_pain'] === 'asymptomatic') {
			input['chest_pain'] = 'asx'
		}
		let isCorrect = Math.random() > 0.4
		let className = isCorrect
			? input[config.target]
			: config.targetValues[
					Math.floor(Math.random() * config.targetValues.length)
			  ]
		output = {
			className,
			probabilities: {
				['Negative']: className === 'Negative' ? 0.95 : 0.05,
				['Positive']: className === 'Positive' ? 0.95 : 0.05,
			},
		}
	} else if (config.name === 'boston') {
		let value = parseFloat(input[config.target]) + Math.random() * 5
		output = { value }
	} else if (config.name === 'iris') {
		if (!config.targetValues) {
			throw Error()
		}
		let isCorrect = Math.random() > 0.4
		let className = isCorrect
			? input[config.target]
			: config.targetValues[
					Math.floor(Math.random() * config.targetValues.length)
			  ]
		output = {
			className,
			probabilities: {
				['Iris Setosa']: className === 'Iris Setosa' ? 0.95 : 0.025,
				['Iris Virginica']: className === 'Iris Virginica' ? 0.95 : 0.025,
				['Iris Versicolor']: className === 'Iris Versicolor' ? 0.95 : 0.025,
			},
		}
	}

	let prediction: Prediction = {
		date: date.toISOString(),
		identifier: i.toString(),
		input,
		modelId: config.modelId,
		output,
		type: 'prediction',
	}

	await track(prediction)
	if (Math.random() > 0.4) {
		let trueValue: TrueValue = {
			date: date.toISOString(),
			identifier: i,
			modelId: config.modelId,
			trueValue: input[config.target],
			type: 'true_value',
		}
		await track(trueValue)
	}
}
