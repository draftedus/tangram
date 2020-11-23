import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
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
	LinearRegressor = "linear_regressor",
	TreeRegressor = "tree_regressor",
	LinearBinaryClassifier = "linear_binary_classifier",
	TreeBinaryClassifier = "tree_binary_classifier",
	LinearMulticlassClassifier = "linear_multiclass_classifier",
	TreeMulticlassClassifier = "tree_multiclass_classifier",
}

export type FeatureImportance = {
	featureImportanceValue: number
	featureName: string
}

export type LinearRegressorProps = {
	featureImportances: FeatureImportance[]
	nFeatures: number
}

export type LinearBinaryClassifierProps = {
	featureImportances: FeatureImportance[]
	nFeatures: number
}

export type LinearMulticlassClassifierProps = {
	featureImportances: FeatureImportance[]
	nFeatures: number
}

export type TreeRegressorProps = {
	featureImportances: FeatureImportance[]
	nFeatures: number
}

export type TreeBinaryClassifierProps = {
	featureImportances: FeatureImportance[]
	nFeatures: number
}

export type TreeMulticlassClassifierProps = {
	featureImportances: FeatureImportance[]
	nFeatures: number
}
