let fetch = require('node-fetch')
let native = require('./build/Release/native.node')

export type PredictOptions = {
	threshold?: number
}

export type Input = {
	[key: string]: string | number | boolean | null | undefined
}

export type RegressionOutput = {
	value: number
}

export type ClassificationOutput<Input = string, Classes = string> = {
	className: Classes
	probabilities: { [K in keyof Classes]: number }
	shapValues: { [K in keyof Input]: number }
}

export type Output = RegressionOutput | ClassificationOutput

export type MonitorEvent = PredictionMonitorEvent | TrueValueMonitorEvent

export type PredictionMonitorEvent = {
	identifier?: number | string
	input: { [key: string]: string | number | boolean | null | undefined }
	modelId: string
	output: ClassificationOutput | RegressionOutput
	type: 'prediction'
}

export type TrueValueMonitorEvent = {
	identifier: number | string
	modelId: string
	trueValue: number | string
	type: 'true_value'
}

export type ModelOptions = {
	tangramUrl?: string
}

type LogPredictionOptions = {
	identifier?: string
	input: Input
	options?: PredictOptions
	output: Output
}

type LogTrueValueOptions = {
	identifier: string
	trueValue: string | number
}

export class Model<InputType extends Input, OutputType extends Output> {
	private model: unknown

	private tangramUrl: string
	private logQueue: MonitorEvent[]

	constructor(data: ArrayBuffer, options?: ModelOptions) {
		this.model = native.model_load(data)
		this.tangramUrl = options?.tangramUrl ?? 'https://app.tangramhq.com.com'
		this.logQueue = []
	}

	public id(): string {
		return native.model_id(this.model)
	}

	public predictSync<PredictInput extends InputType | InputType[]>(
		input: PredictInput,
		options?: PredictOptions,
	): PredictInput extends InputType[] ? OutputType[] : OutputType {
		let isArray = Array.isArray(input)
		let inputJson = JSON.stringify(isArray ? input : [input])
		let optionsJson = options ? JSON.stringify(options) : undefined
		let outputJson = native.model_predict(this.model, inputJson, optionsJson)
		let output = JSON.parse(outputJson)
		output = isArray ? output : output[0]
		return output as PredictInput extends InputType[]
			? OutputType[]
			: OutputType
	}

	public async logPrediction(options: LogPredictionOptions): Promise<void> {
		this.logEvent({
			modelId: this.id(),
			type: 'prediction' as const,
			...options,
		})
	}

	public enqueueLogPrediction(options: LogPredictionOptions) {
		this.logQueue.push({
			modelId: this.id(),
			type: 'prediction' as const,
			...options,
		})
	}

	public async logTrueValue(options: LogTrueValueOptions): Promise<void> {
		this.logEvent({
			modelId: this.id(),
			type: 'true_value' as const,
			...options,
		})
	}

	public enqueueLogTrueValue(options: LogTrueValueOptions) {
		this.logQueue.push({
			modelId: this.id(),
			type: 'true_value' as const,
			...options,
		})
	}

	public async flushLogQueue(): Promise<void> {
		await this.logEvents(this.logQueue)
		this.logQueue = []
	}

	private async logEvent(event: MonitorEvent): Promise<void> {
		await this.logEvents([event])
	}

	private async logEvents(events: MonitorEvent[]): Promise<void> {
		let url = this.tangramUrl + '/track'
		let body = JSON.stringify(events)
		if (typeof fetch === 'undefined') {
			throw Error('Tangram cannot find the fetch function.')
		}
		let response = await fetch(url, {
			body,
			headers: {
				'Content-Type': 'application/json',
			},
			method: 'POST',
		})
		if (!response.ok) {
			throw Error(await response.text())
		}
	}
}
