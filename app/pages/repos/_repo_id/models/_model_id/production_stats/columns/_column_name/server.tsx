import { Enum, Props as EnumProps } from './enum'
import { Number, Props as NumberProps } from './number'
import { Text, Props as TextProps } from './text'
import { renderPage } from 'common/render'
import { DateWindow, DateWindowSelectField } from 'common/time'
import { PinwheelInfo, h, ui } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
export type { Props as EnumProps } from './enum'

export type Props = {
	columnName: string
	dateWindow: DateWindow
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
	Enum = 'enum',
	Number = 'number',
	Text = 'text',
}

export default function ProductionStatsColumnsPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Number:
			inner = <Number {...props.inner.value} />
			break
		case Type.Enum:
			inner = <Enum {...props.inner.value} />
			break
		case Type.Text:
			inner = <Text {...props.inner.value} />
			break
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.ProductionStats}
		>
			<ui.S1>
				<ui.H1>{props.columnName}</ui.H1>
				<DateWindowSelectField dateWindow={props.dateWindow} />
				{inner}
			</ui.S1>
		</ModelLayout>,
	)
}
