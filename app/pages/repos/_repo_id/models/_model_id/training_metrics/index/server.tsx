import { BinaryClassifierTrainingMetricsIndexPage } from './binary_classifier'
import { MulticlassClassifierTrainingMetricsIndexPage } from './multiclass_classifier'
import { Props, Type } from './props'
import { RegressorTrainingMetricsIndexPage } from './regressor'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default function TrainingMetricsIndexPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Regressor:
			inner = <RegressorTrainingMetricsIndexPage {...props.inner.value} />
			break
		case Type.BinaryClassifier:
			inner = (
				<BinaryClassifierTrainingMetricsIndexPage {...props.inner.value} />
			)
			break
		case Type.MulticlassClassifier:
			inner = (
				<MulticlassClassifierTrainingMetricsIndexPage {...props.inner.value} />
			)
			break
	}

	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.TrainingMetrics}
		>
			{inner}
		</ModelLayout>,
	)
}
