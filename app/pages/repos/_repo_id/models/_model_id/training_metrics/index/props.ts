import { ModelLayoutInfo } from 'layouts/model_layout'

export enum Type {
	Regressor = 'regressor',
	BinaryClassifier = 'binary_classifier',
	MulticlassClassifier = 'multiclass_classifier',
}

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
}

export type Inner =
	| {
			type: Type.Regressor
			value: RegressorProps
	  }
	| {
			type: Type.BinaryClassifier
			value: BinaryClassifierProps
	  }
	| {
			type: Type.MulticlassClassifier
			value: MulticlassClassifierProps
	  }

export type RegressorProps = {
	baselineMse: number
	baselineRmse: number
	id: string
	mse: number
	rmse: number
}

export type BinaryClassifierProps = {
	accuracy: number
	aucRoc: number
	baselineAccuracy: number
	classes: string[]
	id: string
	precision: number
	recall: number
}

export type MulticlassClassifierProps = {
	accuracy: number
	baselineAccuracy: number
	classMetrics: Array<{
		precision: number
		recall: number
	}>
	classes: string[]
	id: string
}
