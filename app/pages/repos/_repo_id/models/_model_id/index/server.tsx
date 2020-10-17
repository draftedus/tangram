import {
	MulticlassClassifierIndexPage,
	Props as MulticlassClassifierProps,
} from './classifier'
import { RegressorIndexPage, Props as RegressorProps } from './regressor'
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

export enum Type {
	Regressor = 'regressor',
	MulticlassClassifier = 'multiclass_classifier',
}

export type Inner =
	| { type: Type.Regressor; value: RegressorProps }
	| { type: Type.MulticlassClassifier; value: MulticlassClassifierProps }

export default function ModelIndexPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Regressor: {
			inner = <RegressorIndexPage {...props.inner.value} />
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
