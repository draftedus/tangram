import { BinaryClassifierIndexPage } from './binary_classifier'
import { MulticlassClassifierIndexPage } from './multiclass_classifier'
import { Props, Type } from './props'
import { RegressorIndexPage } from './regressor'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default function ModelIndexPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Regressor: {
			inner = <RegressorIndexPage {...props.inner.value} />
			break
		}
		case Type.BinaryClassifier: {
			inner = <BinaryClassifierIndexPage {...props.inner.value} />
			break
		}
		case Type.MulticlassClassifier: {
			inner = <MulticlassClassifierIndexPage {...props.inner.value} />
			break
		}
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Overview}
		>
			{inner}
		</ModelLayout>,
	)
}
