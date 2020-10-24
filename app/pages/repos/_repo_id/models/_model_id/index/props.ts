import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
}

export enum Type {
	Regressor = 'regressor',
	BinaryClassifier = 'binary_classifier',
	MulticlassClassifier = 'multiclass_classifier',
}

export type Inner =
	| { type: Type.Regressor; value: RegressorProps }
	| { type: Type.BinaryClassifier; value: BinaryClassifierProps }
	| { type: Type.MulticlassClassifier; value: MulticlassClassifierProps }

export type RegressorProps = {
	id: string
	lossesChartData: number[]
	metrics: {
		baselineMse: number
		baselineRmse: number
		mse: number
		rmse: number
	}
	trainingSummary: {
		chosenModelTypeName: string
		columnCount: number
		modelComparisonMetricTypeName: string
		targetColumn: string
		testFraction: number
		testRowCount: number
		trainRowCount: number
	}
}

export type BinaryClassifierProps = {
	id: string
	lossesChartData: number[]
	metrics: {
		accuracy: number
		aucRoc: number
		baselineAccuracy: number
		precision: number
		recall: number
	}
	title: string
	trainingSummary: {
		chosenModelTypeName: string
		columnCount: number
		modelComparisonMetricTypeName: string
		testRowCount: number
		trainRowCount: number
	}
}

export type MulticlassClassifierProps = {
	id: string
	lossesChartData: number[]
	metrics: {
		accuracy: number
		baselineAccuracy: number
		classMetrics: Array<{
			precision: number
			recall: number
		}>
		classes: string[]
	}
	title: string
	trainingSummary: {
		chosenModelTypeName: string
		columnCount: number
		modelComparisonMetricTypeName: string
		testRowCount: number
		trainRowCount: number
	}
}
