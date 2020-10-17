import {
	MulticlassClassifierProductionMetricsIndexPage,
	Props as MulticlassClassifierProps,
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
export type { Props as MulticlassClassifierProps } from './classifier'

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
			type: Type.MulticlassClassifier
			value: MulticlassClassifierProps
	  }

export enum Type {
	Regressor = 'regressor',
	MulticlassClassifier = 'multiclass_classifier',
}

export default function ProductionMetricsPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Regressor: {
			inner = <RegressorProductionMetricsPage {...props.inner.value} />
			break
		}
		case Type.MulticlassClassifier: {
			inner = (
				<MulticlassClassifierProductionMetricsIndexPage
					{...props.inner.value}
				/>
			)
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
