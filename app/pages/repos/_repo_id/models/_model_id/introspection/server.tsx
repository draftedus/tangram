import { LinearBinaryClassifierModelPage } from './linear_binary_classifier'
import { LinearMulticlassClassifierModelPage } from './linear_multiclass_classifier'
import { LinearRegressorModelPage } from './linear_regressor'
import { Props, Type } from './props'
import { TreeBinaryClassifierModelPage } from './tree_binary_classifier'
import { TreeMulticlassClassifierModelPage } from './tree_multiclass_classifier'
import { TreeRegressorModelPage } from './tree_regressor'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

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
