import {
	ClassifierProductionMetricsIndexPage,
	Props as ClassifierProps,
} from './classifier'
import {
	RegressorProductionMetricsPage,
	Props as RegresssorProps,
} from './regressor'
import { PinwheelInfo, assert, h, renderPage } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
export type { Props as RegressorProps } from './regressor'
export type { Props as ClassifierProps } from './classifier'

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
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
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.ProductionMetrics}
		>
			{inner}
		</ModelLayout>,
	)
}
