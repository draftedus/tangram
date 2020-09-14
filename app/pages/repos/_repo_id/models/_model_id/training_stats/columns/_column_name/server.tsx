import { EnumColumnDetail, Props as EnumProps } from './enum'
import { NumberColumnDetail, Props as NumberProps } from './number'
import { TextColumnDetail, Props as TextProps } from './text'
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

export type Inner =
	| {
			type: Type.Number
			value: NumberProps
	  }
	| {
			type: Type.Enum
			value: EnumProps
	  }
	| {
			type: Type.Text
			value: TextProps
	  }

export enum Type {
	Number = 'number',
	Enum = 'enum',
	Text = 'text',
}

export default function TrainingStatsColumnPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Number:
			inner = <NumberColumnDetail {...props.inner.value} />
			break
		case Type.Enum:
			inner = <EnumColumnDetail {...props.inner.value} />
			break
		case Type.Text:
			inner = <TextColumnDetail {...props.inner.value} />
			break
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.TrainingStats}
		>
			{inner}
		</ModelLayout>,
	)
}
