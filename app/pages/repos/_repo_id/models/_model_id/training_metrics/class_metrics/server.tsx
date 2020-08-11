import {
	BinaryClassifierClassMetricsPage,
	Props as BinaryClassifierProps,
} from './binary_classifier'
import {
	MulticlassClassifierClassMetricsPage,
	Props as MulticlassClassifierProps,
} from './multiclass_classifier'
import { PinwheelInfo, h, renderPage } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	info: PinwheelInfo
	inner: Inner
	modelLayoutProps: ModelLayoutProps
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
		<ModelLayout {...props.modelLayoutProps} info={props.info}>
			{inner}
		</ModelLayout>,
	)
}
