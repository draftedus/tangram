import {
	BinaryClassifierClassMetricsPage,
	Props as BinaryClassifierProps,
} from './binary_classifier'
import {
	MulticlassClassifierClassMetricsPage,
	Props as MulticlassClassifierProps,
} from './multiclass_classifier'
import { h } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	inner: Inner
	modelLayout: ModelLayoutProps
}

export enum ModelType {
	BinaryClassifier = 'binaryClassifier',
	MulticlassClassifier = 'multiclassClassifier',
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
	return <ModelLayout {...props.modelLayout}>{inner}</ModelLayout>
}
