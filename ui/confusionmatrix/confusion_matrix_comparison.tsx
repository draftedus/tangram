import { Children, css, cssClass, cx, h, useCss } from '../deps'
import { border, mobile, variables } from '../theme'
import { Token } from '../token'
import { config } from './config'

// |-----------------------------------------------------------|
// |           ||       |                Actual                |
// |===========||==============================================|
// |           ||       |       True       |      False        |
// |           ||----------------------------------------------|
// |           ||       |                  |                   |
// |           || True  |  True Positives  |  False Positives  |
// |           ||       |                  |                   |
// | Predicted ||-------|--------------------------------------|
// |           ||       |                  |                   |
// |           || False |  False Negatives |  True Negatives   |
// |           ||       |                  |                   |
// |-----------------------------------------------------------|

type ConfusionMatrixComparisonProps = {
	classLabel: string
	colorA: string
	colorB: string
	textColorA: string
	textColorB: string
	valueA: {
		falseNegative: number | null
		falsePositive: number | null
		trueNegative: number | null
		truePositive: number | null
	}
	valueATitle: string
	valueATitleColor: string
	valueB: {
		falseNegative: number | null
		falsePositive: number | null
		trueNegative: number | null
		truePositive: number | null
	}
	valueBTitle: string
	valueBTitleColor: string
	valueFormatter?: (value: number) => string
}

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: {
		display: 'grid',
		grid: 'auto 1fr 1fr / auto 1fr 1fr',
		gridGap: '1rem',
		gridTemplateAreas: `"empty actual-true-label actual-false-label" "predicted-true-label true-positive false-positive" "predicted-false-label false-negative true-negative"`,
		maxWidth: '100%',
		overflow: 'auto',
	},
})

export function ConfusionMatrixComparison(
	props: ConfusionMatrixComparisonProps,
) {
	useCss(wrapperCss)
	return (
		<div class={wrapperClass}>
			<ConfusionMatrixLabel area="actual-true-label">
				<div>Actual</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="actual-false-label">
				<div>Actual Not</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-true-label" left>
				<div>Predicted</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-false-label" left>
				<div>Predicted Not</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixComparisonItem
				area="true-positive"
				colorA={props.colorA}
				colorB={props.colorB}
				label="True Positives"
				textColorA={props.textColorA}
				textColorB={props.textColorB}
				true
				valueA={props.valueA.truePositive}
				valueATitle={props.valueATitle}
				valueB={props.valueB.truePositive}
				valueBTitle={props.valueBTitle}
				valueFormatter={props.valueFormatter}
			/>
			<ConfusionMatrixComparisonItem
				area="false-positive"
				colorA={props.colorA}
				colorB={props.colorB}
				label="False Positives"
				textColorA={props.textColorA}
				textColorB={props.textColorB}
				valueA={props.valueA.falsePositive}
				valueATitle={props.valueATitle}
				valueB={props.valueB.falsePositive}
				valueBTitle={props.valueBTitle}
				valueFormatter={props.valueFormatter}
			/>
			<ConfusionMatrixComparisonItem
				area="false-negative"
				colorA={props.colorA}
				colorB={props.colorB}
				label="False Negatives"
				textColorA={props.textColorA}
				textColorB={props.textColorB}
				valueA={props.valueA.falseNegative}
				valueATitle={props.valueATitle}
				valueB={props.valueB.falseNegative}
				valueBTitle={props.valueBTitle}
				valueFormatter={props.valueFormatter}
			/>
			<ConfusionMatrixComparisonItem
				area="true-negative"
				colorA={props.colorA}
				colorB={props.colorB}
				label="True Negatives"
				textColorA={props.textColorA}
				textColorB={props.textColorB}
				true
				valueA={props.valueA.trueNegative}
				valueATitle={props.valueATitle}
				valueB={props.valueB.trueNegative}
				valueBTitle={props.valueBTitle}
				valueFormatter={props.valueFormatter}
			/>
		</div>
	)
}

type ConfusionMatrixItemProps = {
	area: string
	colorA: string
	colorB: string
	label: string
	textColorA: string
	textColorB: string
	true?: boolean
	valueA: number | null
	valueATitle: string
	valueB: number | null
	valueBTitle: string
	valueFormatter?: (value: number) => string
}

let itemWrapperClass = cssClass()
let itemWrapperCss = css({
	[`.${itemWrapperClass}`]: {
		alignContent: 'center',
		border,
		borderRadius: config.borderRadius,
		display: 'grid',
		gridGap: '1rem',
		justifyContent: 'center',
		justifyItems: 'center',
		padding: '2rem',
		textAlign: 'center',
	},
})

let trueItemWrapperClass = cssClass()
let trueItemWrapperCss = css({
	[`.${trueItemWrapperClass}`]: {
		backgroundColor: config.trueBackgroundColor,
		color: config.trueForegroundColor,
	},
})

let falseItemWrapperClass = cssClass()
let falseItemWrapperCss = css({
	[`.${falseItemWrapperClass}`]: {
		backgroundColor: config.falseBackgroundColor,
		color: config.falseForegroundColor,
	},
})

let numberComparisonContainerClass = cssClass()
let numberComparisonContainerCss = css({
	[`.${numberComparisonContainerClass}`]: {
		alignItems: 'center',
		display: 'grid',
		gridColumnGap: '2rem',
		gridRowGap: '.5rem',
		gridTemplateColumns: 'auto auto',
		justifyContent: 'center',
		justifyItems: 'center',
	},
})

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: { fontSize: '1.25rem' },
})

let valueClass = cssClass()
let valueCss = css({
	[`.${valueClass}`]: {
		fontSize: '2rem',
	},
	[`${mobile}`]: {
		[`.${valueClass}`]: {
			fontSize: '1.5rem',
		},
	},
})
function ConfusionMatrixComparisonItem(props: ConfusionMatrixItemProps) {
	useCss(
		itemWrapperCss,
		trueItemWrapperCss,
		falseItemWrapperCss,
		numberComparisonContainerCss,
		titleCss,
		valueCss,
	)
	let valueFormatter = props.valueFormatter || (v => v.toString())
	let wrapperStyle = {
		gridArea: props.area,
	}
	return (
		<div
			class={cx(
				wrapperClass,
				props.true ? trueItemWrapperClass : falseItemWrapperClass,
			)}
			style={wrapperStyle}
		>
			<div class={titleClass}>{props.label}</div>
			<div class={numberComparisonContainerClass}>
				<div class={valueClass}>
					{props.valueA === null ? 'N/A' : valueFormatter(props.valueA)}
				</div>
				<div class={valueClass}>
					{props.valueB === null ? 'N/A' : valueFormatter(props.valueB)}
				</div>
				<div>
					<Token color={props.colorA} textColor={props.textColorA}>
						{props.valueATitle}
					</Token>
				</div>
				<div>
					<Token color={props.colorB} textColor={props.textColorB}>
						{props.valueBTitle}
					</Token>
				</div>
			</div>
		</div>
	)
}

type ConfusionMatrixLabelProps = {
	area: string
	children?: Children
	left?: boolean
}

let labelClass = cssClass()
let labelCss = css({
	[`.${labelClass}`]: {
		alignSelf: 'center',
		display: 'grid',
		fontWeight: 'bold',
		gridGap: '0.5rem',
		textAlign: 'center',
	},
})

function ConfusionMatrixLabel(props: ConfusionMatrixLabelProps) {
	useCss(labelCss)
	let style = {
		gridArea: props.area,
		justifyItems: props.left ? 'end' : 'center',
	}
	return (
		<div class={labelClass} style={style}>
			{props.children}
		</div>
	)
}
