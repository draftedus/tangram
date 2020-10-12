import { Enum, Props as EnumProps } from './enum'
import { Number, Props as NumberProps } from './number'
import { Text, Props as TextProps } from './text'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { DateWindow } from 'common/time'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { h } from 'preact'
export type { Props as EnumProps } from './enum'
import { DateWindowSelectField } from 'common/date_window_select_field'

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
				<ui.Form>
					<DateWindowSelectField dateWindow={props.dateWindow} />
					<noscript>
						<ui.Button>{'Submit'}</ui.Button>
					</noscript>
				</ui.Form>
				{inner}
			</ui.S1>
		</ModelLayout>,
	)
}
