import {
	ClassifierProductionMetricsIndexPage,
	Props as ClassifierProps,
} from './classifier'
import {
	RegressorProductionMetricsPage,
	Props as RegresssorProps,
} from './regressor'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import { renderPage } from 'common/render'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { h } from 'preact'
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
