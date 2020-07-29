let textEncoder = new TextEncoder()
let textDecoder = new TextDecoder()

let libtangram: any

let cachedMemory: Uint8Array | undefined
function memory(): Uint8Array {
	if (!cachedMemory || cachedMemory.buffer !== libtangram.memory.buffer) {
		cachedMemory = new Uint8Array(libtangram.memory.buffer)
	}
	return cachedMemory
}

function allocPointer(): number {
	return libtangram.tangram_alloc(4)
}

function readPointer(ptr: number) {
	return new DataView(memory().buffer, ptr, 4).getUint32(0, true)
}

function sendString(s: string): number {
	let bytes = textEncoder.encode(s)
	let len = bytes.byteLength
	let ptr = libtangram.tangram_alloc(len + 1)
	memory().set(bytes, ptr)
	memory()[ptr + len] = 0
	return ptr
}

function recvString(ptr: number): string {
	let len = 0
	while (memory()[ptr + len] !== 0) {
		len++
	}
	let bytes = memory().subarray(ptr, ptr + len)
	let s = textDecoder.decode(bytes)
	return s
}

export type PredictOptions = {
	threshold?: number
}

export type Input = {
	[key: string]: string | number | boolean | null | undefined
}

export type RegressionOutput = {
	value: number
}

export type ClassificationOutput<Classes = string> = {
	className: Classes
	probabilities: { [K in keyof Classes]: number }
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
		let dataLen = data.byteLength
		let dataPtr = libtangram.tangram_alloc(dataLen)
		memory().set(new Uint8Array(data), dataPtr)
		let modelPtrPtr = allocPointer()
		let result = libtangram.tangram_model_load(dataPtr, dataLen, modelPtrPtr)
		if (result != 0) {
			throw Error()
		}
		libtangram.tangram_dealloc(dataPtr)
		let modelPtr = readPointer(modelPtrPtr)
		libtangram.tangram_dealloc(modelPtrPtr)
		this.model = modelPtr
		this.tangramUrl = options?.tangramUrl ?? 'https://app.tangramhq.com'
		this.logQueue = []
	}

	public id(): string {
		let outputPtrPtr = allocPointer()
		let result = libtangram.tangram_model_id(this.model, outputPtrPtr)
		if (result != 0) {
			throw Error()
		}
		let outputPtr = readPointer(outputPtrPtr)
		libtangram.tangram_dealloc(outputPtrPtr)
		let id = recvString(outputPtr)
		libtangram.tangram_string_free(outputPtr)
		return id
	}

	public predictSync<PredictInput extends InputType | InputType[]>(
		input: PredictInput,
		options?: PredictOptions,
	): PredictInput extends InputType[] ? OutputType[] : OutputType {
		let isArray = Array.isArray(input)
		let inputPtr = sendString(JSON.stringify(isArray ? input : [input]))
		let optionsPtr = options && sendString(JSON.stringify(options))
		let outputPtrPtr = allocPointer()
		let result = libtangram.tangram_model_predict(
			this.model,
			inputPtr,
			optionsPtr,
			outputPtrPtr,
		)
		if (result != 0) {
			throw Error()
		}
		libtangram.tangram_dealloc(inputPtr)
		let outputPtr = readPointer(outputPtrPtr)
		libtangram.tangram_dealloc(outputPtrPtr)
		let outputJson = recvString(outputPtr)
		libtangram.tangram_string_free(outputPtr)
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
		if (!fetch) {
			throw Error(
				'Tangram cannot find the fetch function. Please install an appropriate polyfill for your environment.',
			)
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
