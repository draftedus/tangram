import { EnumColumnDetail, Props as EnumProps } from './enum'
import { NumberColumnDetail, Props as NumberProps } from './number'
import { TextColumnDetail, Props as TextProps } from './text'
import { h } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	inner: Inner
	modelLayout: ModelLayoutProps
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
	return <ModelLayout {...props.modelLayout}>{inner}</ModelLayout>
}
