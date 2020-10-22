import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ModelLayoutInfo } from 'layouts/model_layout'

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
}

export type Inner =
	| {
			type: Type.LinearRegressor
			value: LinearRegressorProps
	  }
	| {
			type: Type.LinearBinaryClassifier
			value: LinearBinaryClassifierProps
	  }
	| {
			type: Type.LinearMulticlassClassifier
			value: LinearMulticlassClassifierProps
	  }
	| {
			type: Type.TreeRegressor
			value: TreeRegressorProps
	  }
	| {
			type: Type.TreeBinaryClassifier
			value: TreeBinaryClassifierProps
	  }
	| {
			type: Type.TreeMulticlassClassifier
			value: TreeMulticlassClassifierProps
	  }

export enum Type {
	LinearRegressor = 'linear_regressor',
	TreeRegressor = 'tree_regressor',
	LinearBinaryClassifier = 'linear_binary_classifier',
	TreeBinaryClassifier = 'tree_binary_classifier',
	LinearMulticlassClassifier = 'linear_multiclass_classifier',
	TreeMulticlassClassifier = 'tree_multiclass_classifier',
}

export type LinearRegressorProps = {
	featureImportances: Array<[string, number]>
}

export type LinearBinaryClassifierProps = {
	featureImportances: Array<[string, number]>
}

export type LinearMulticlassClassifierProps = {
	featureImportances: Array<[string, number]>
}

export type TreeRegressorProps = {
	featureImportances: Array<[string, number]>
}

export type TreeBinaryClassifierProps = {
	featureImportances: Array<[string, number]>
}

export type TreeMulticlassClassifierProps = {
	featureImportances: Array<[string, number]>
}
