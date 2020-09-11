import { renderPage } from 'common/render'
import { PinwheelInfo, h } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'

import { LinearBinaryClassifierModelPage } from './linear_binary_classifier'
import { LinearMulticlassClassifierModelPage } from './linear_multiclass_classifier'
import { LinearRegressorModelPage } from './linear_regressor'
import { TreeBinaryClassifierModelPage } from './tree_binary_classifier'
import { TreeMulticlassClassifierModelPage } from './tree_multiclass_classifier'
import { TreeRegressorModelPage } from './tree_regressor'

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
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
				| Type.TreeRegressor
				| Type.TreeBinaryClassifier
				| Type.TreeMulticlassClassifier
			value: {
				featureImportances: Array<[string, number]>
			}
	  }

export enum Type {
	LinearRegressor = 'linearRegressor',
	TreeRegressor = 'treeRegressor',
	LinearBinaryClassifier = 'linearBinaryClassifier',
	TreeBinaryClassifier = 'treeBinaryClassifier',
	LinearMulticlassClassifier = 'linearMulticlassClassifier',
	TreeMulticlassClassifier = 'treeMulticlassClassifier',
}

export default function IntrospectionPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.LinearRegressor: {
			inner = <LinearRegressorModelPage {...props.inner.value} />
			break
		}
		case Type.TreeRegressor: {
			inner = <TreeRegressorModelPage {...props.inner.value} />
			break
		}
		case Type.LinearBinaryClassifier: {
			inner = <LinearBinaryClassifierModelPage {...props.inner.value} />
			break
		}
		case Type.TreeBinaryClassifier: {
			inner = <TreeBinaryClassifierModelPage {...props.inner.value} />
			break
		}
		case Type.LinearMulticlassClassifier: {
			inner = <LinearMulticlassClassifierModelPage {...props.inner.value} />
			break
		}
		case Type.TreeMulticlassClassifier: {
			inner = <TreeMulticlassClassifierModelPage {...props.inner.value} />
			break
		}
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Introspection}
		>
			{inner}
		</ModelLayout>,
	)
}
