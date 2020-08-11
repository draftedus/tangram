import {
	ClassifierProductionMetricsIndexPage,
	Props as ClassifierProps,
} from './classifier'
import {
	RegressorProductionMetricsPage,
	Props as RegresssorProps,
} from './regressor'
import { PinwheelInfo, assert, h, renderPage } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'
export type { Props as RegressorProps } from './regressor'
export type { Props as ClassifierProps } from './classifier'

export type Props = {
	info: PinwheelInfo
	inner: Inner
	modelLayoutProps: ModelLayoutProps
}

export type Inner =
	| {
			type: Type.Regressor
			value: RegresssorProps
	  }
	| {
			type: Type.Classifier
			value: ClassifierProps
	  }

export enum Type {
	Classifier = 'classifier',
	Regressor = 'regressor',
}

export default function ProductionMetricsPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Regressor: {
			assert(props.inner.value)
			inner = <RegressorProductionMetricsPage {...props.inner.value} />
			break
		}
		case Type.Classifier: {
			inner = <ClassifierProductionMetricsIndexPage {...props.inner.value} />
			break
		}
	}
	return renderPage(
		<ModelLayout {...props.modelLayoutProps} info={props.info}>
			{inner}
		</ModelLayout>,
	)
}
