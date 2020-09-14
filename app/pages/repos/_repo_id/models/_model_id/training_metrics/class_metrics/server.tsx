import {
	BinaryClassifierClassMetricsPage,
	Props as BinaryClassifierProps,
} from './binary_classifier'
import {
	MulticlassClassifierClassMetricsPage,
	Props as MulticlassClassifierProps,
} from './multiclass_classifier'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import { renderPage } from 'common/render'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { h } from 'preact'

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
}

export enum ModelType {
	BinaryClassifier = 'BinaryClassifier',
	MulticlassClassifier = 'MulticlassClassifier',
}

export type Inner =
	| {
			type: ModelType.BinaryClassifier
			value: BinaryClassifierProps
	  }
	| {
			type: ModelType.MulticlassClassifier
			value: MulticlassClassifierProps
	  }

export default function ClassMetricsPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case ModelType.BinaryClassifier:
			inner = <BinaryClassifierClassMetricsPage {...props.inner.value} />
			break
		case ModelType.MulticlassClassifier:
			inner = <MulticlassClassifierClassMetricsPage {...props.inner.value} />
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
