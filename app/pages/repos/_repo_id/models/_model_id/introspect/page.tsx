import { h } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

import { GBTBinaryClassifierModelPage } from './gbt_binary_classifier'
import { GBTMulticlassClassifierModelPage } from './gbt_multiclass_classifier'
import { GBTRegressorModelPage } from './gbt_regressor'
import { LinearBinaryClassifierModelPage } from './linear_binary_classifier'
import { LinearMulticlassClassifierModelPage } from './linear_multiclass_classifier'
import { LinearRegressorModelPage } from './linear_regressor'

export type Props = {
	inner: Inner
	modelLayout: ModelLayoutProps
}

export type Inner =
	| {
			type: Type.LinearRegressor
			value: {
				bias: number
				targetColumnName: string
				weights: Array<[string, number]>
			}
	  }
	| {
			type: Type.LinearBinaryClassifier
			value: {
				bias: number
				positiveClassName: string
				targetColumnName: string
				weights: Array<[string, number]>
			}
	  }
	| {
			type: Type.LinearMulticlassClassifier
			value: {
				biases: number[]
				classes: string[]
				selectedClass: string
				targetColumnName: string
				weights: Array<Array<[string, number]>>
			}
	  }
	| {
			type:
				| Type.GBTRegressor
				| Type.GBTBinaryClassifier
				| Type.GBTMulticlassClassifier
			value: {
				featureImportances: Array<[string, number]>
			}
	  }

export enum Type {
	LinearRegressor = 'linearRegressor',
	GBTRegressor = 'gbtRegressor',
	LinearBinaryClassifier = 'linearBinaryClassifier',
	GBTBinaryClassifier = 'gbtBinaryClassifier',
	LinearMulticlassClassifier = 'linearMulticlassClassifier',
	GBTMulticlassClassifier = 'gbtMulticlassClassifier',
}

export default function ModelPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.LinearRegressor: {
			inner = <LinearRegressorModelPage {...props.inner.value} />
			break
		}
		case Type.GBTRegressor: {
			inner = <GBTRegressorModelPage {...props.inner.value} />
			break
		}
		case Type.LinearBinaryClassifier: {
			inner = <LinearBinaryClassifierModelPage {...props.inner.value} />
			break
		}
		case Type.GBTBinaryClassifier: {
			inner = <GBTBinaryClassifierModelPage {...props.inner.value} />
			break
		}
		case Type.LinearMulticlassClassifier: {
			inner = <LinearMulticlassClassifierModelPage {...props.inner.value} />
			break
		}
		case Type.GBTMulticlassClassifier: {
			inner = <GBTMulticlassClassifierModelPage {...props.inner.value} />
			break
		}
	}
	return <ModelLayout {...props.modelLayout}>{inner}</ModelLayout>
}
