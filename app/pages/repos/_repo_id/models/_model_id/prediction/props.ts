import { FeatureContributionsChartData } from '@tangramhq/charts'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Column = UnknownColumn | NumberColumn | EnumColumn | TextColumn

export type Props = {
	columns: Column[]
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	prediction: Prediction | null
}

export enum ColumnType {
	Unknown = 'unknown',
	Number = 'number',
	Enum = 'enum',
	Text = 'text',
}

export type UnknownColumn = {
	name: string
	type: ColumnType.Unknown
	value: string | null
}

export type NumberColumn = {
	max: number
	min: number
	name: string
	p25: number
	p50: number
	p75: number
	type: ColumnType.Number
	value: string | null
}

export type EnumColumn = {
	histogram: Array<[string, number]>
	name: string
	options: string[]
	type: ColumnType.Enum
	value: string | null
}

export type TextColumn = {
	name: string
	type: ColumnType.Text
	value: string | null
}

export enum PredictionType {
	Regression = 'regression',
	BinaryClassification = 'binary_classification',
	MulticlassClassification = 'multiclass_classification',
}

export type Prediction =
	| {
			type: PredictionType.Regression
			value: RegressionPrediction
	  }
	| {
			type: PredictionType.BinaryClassification
			value: BinaryClassificationPrediction
	  }
	| {
			type: PredictionType.MulticlassClassification
			value: MulticlassClassificationPrediction
	  }

export type RegressionPrediction = {
	featureContributionsChartData: FeatureContributionsChartData
	value: number
}

export type BinaryClassificationPrediction = {
	className: string
	classes: string[]
	featureContributionsChartData: FeatureContributionsChartData
	probabilities: Array<[string, number]>
	probability: number
}

export type MulticlassClassificationPrediction = {
	className: string
	classes: string[]
	featureContributionsChartData: FeatureContributionsChartData
	probabilities: Array<[string, number]>
	probability: number
}
