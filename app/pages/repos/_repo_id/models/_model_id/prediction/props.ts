import { PinwheelInfo } from '@tangramhq/pinwheel'
import { InputTable, Prediction } from 'common/predict'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Column = UnknownField | NumberField | EnumField | TextField

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
}

export enum InnerType {
	PredictionForm = 'prediction_form',
	PredictionResult = 'prediction_result',
}

export type Inner =
	| {
			type: InnerType.PredictionForm
			value: PredictionForm
	  }
	| {
			type: InnerType.PredictionResult
			value: PredictionResult
	  }

export type PredictionForm = {
	form: Form
}

export type Form = { fields: Column[] }

export type PredictionResult = {
	inputTable: InputTable
	prediction: Prediction
}

export enum FieldType {
	Unknown = 'unknown',
	Number = 'number',
	Enum = 'enum',
	Text = 'text',
}

export type UnknownField = {
	name: string
	type: FieldType.Unknown
	value: string | null
}

export type NumberField = {
	max: number
	min: number
	name: string
	p25: number
	p50: number
	p75: number
	type: FieldType.Number
	value: string | null
}

export type EnumField = {
	histogram: Array<[string, number]>
	name: string
	options: string[]
	type: FieldType.Enum
	value: string | null
}

export type TextField = {
	name: string
	type: FieldType.Text
	value: string | null
}
