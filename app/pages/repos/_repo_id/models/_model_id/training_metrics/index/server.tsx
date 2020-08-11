import {
	Props as BinaryClassifierProps,
	BinaryClassifierTrainingMetricsIndexPage,
} from './binary_classifier'
import {
	Props as MulticlassClassifierProps,
	MulticlassClassifierTrainingMetricsIndexPage,
} from './multiclass_classifier'
import {
	Props as RegressorProps,
	RegressorTrainingMetricsIndexPage,
} from './regressor'
import { PinwheelInfo, h, renderPage } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'

export enum Type {
	Regressor = 'regressor',
	BinaryClassifier = 'binaryClassifier',
	MulticlassClassifier = 'multiclassClassifier',
}

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
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
