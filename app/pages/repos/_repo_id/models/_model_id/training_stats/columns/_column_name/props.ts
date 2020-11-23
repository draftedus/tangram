import { ModelLayoutInfo } from "layouts/model_layout"

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
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
	Number = "number",
	Enum = "enum",
	Text = "text",
}

export type EnumProps = {
	histogram: Array<[string, number]>
	invalidCount: number
	name: string
	uniqueCount: number
}

export type NumberProps = {
	invalidCount: number
	max: number
	mean: number
	min: number
	name: string
	p25: number
	p50: number
	p75: number
	std: number
	uniqueCount: number
}

export type TextProps = {
	nTokens: number
	name: string
	tokens: TokenStats[]
}

type TokenStats = {
	count: number
	examples_count: number
	token: string
}
