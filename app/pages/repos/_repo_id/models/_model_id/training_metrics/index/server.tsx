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
import { h } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export enum Type {
	Regressor = 'regressor',
	BinaryClassifier = 'binaryClassifier',
	MulticlassClassifier = 'multiclassClassifier',
}

export type Props = {
	inner: Inner
	modelLayoutProps: ModelLayoutProps
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

	return <ModelLayout {...props.modelLayoutProps}>{inner}</ModelLayout>
}
