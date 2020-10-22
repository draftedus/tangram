import { LinearBinaryClassifierTrainingImportancesPage } from './linear_binary_classifier'
import { LinearMulticlassClassifierTrainingImportancesPage } from './linear_multiclass_classifier'
import { LinearRegressorTrainingImportancesPage } from './linear_regressor'
import { Props, Type } from './props'
import { TreeBinaryClassifierTrainingImportancesPage } from './tree_binary_classifier'
import { TreeMulticlassClassifierTrainingImportancesPage } from './tree_multiclass_classifier'
import { TreeRegressorTrainingImportancesPage } from './tree_regressor'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default function TrainingImportancesPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.LinearRegressor: {
			inner = <LinearRegressorTrainingImportancesPage {...props.inner.value} />
			break
		}
		case Type.TreeRegressor: {
			inner = <TreeRegressorTrainingImportancesPage {...props.inner.value} />
			break
		}
		case Type.LinearBinaryClassifier: {
			inner = (
				<LinearBinaryClassifierTrainingImportancesPage {...props.inner.value} />
			)
			break
		}
		case Type.TreeBinaryClassifier: {
			inner = (
				<TreeBinaryClassifierTrainingImportancesPage {...props.inner.value} />
			)
			break
		}
		case Type.LinearMulticlassClassifier: {
			inner = (
				<LinearMulticlassClassifierTrainingImportancesPage
					{...props.inner.value}
				/>
			)
			break
		}
		case Type.TreeMulticlassClassifier: {
			inner = (
				<TreeMulticlassClassifierTrainingImportancesPage
					{...props.inner.value}
				/>
			)
			break
		}
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.TrainingImportances}
		>
			{inner}
		</ModelLayout>,
	)
}
