import { Children, css, cssClass, cx, h, useCss } from '../deps'
import { border, desktop, mobile, variables } from '../theme'
import { Token } from '../token'
import { formatPercent } from '../util'
import { config } from './config'

// |---------------------------------------------------------|
// |           ||     |                Actual                |
// |===========||============================================|
// |           ||     |       Pos        |       Neg         |
// |           ||--------------------------------------------|
// |           ||     |                  |                   |
// |           || Pos |  True Positives  |  False Positives  |
// |           ||     |                  |                   |
// | Predicted ||-----|--------------------------------------|
// |           ||     |                  |                   |
// |           || Neg |  False Negatives |  True Negatives   |
// |           ||     |                  |                   |
// |---------------------------------------------------------|

type ConfusionMatrixProps = {
	classLabel: string
	falseNegatives: number | null
	falsePositives: number | null
	trueNegatives: number | null
	truePositives: number | null
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

export function ConfusionMatrix(props: ConfusionMatrixProps) {
	useCss(wrapperCss)
	let total = null
	if (
		props.truePositives !== null &&
		props.trueNegatives !== null &&
		props.falsePositives !== null &&
		props.falseNegatives !== null
	) {
		total =
			props.truePositives +
			props.trueNegatives +
			props.falsePositives +
			props.falseNegatives
	}
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
			<ConfusionMatrixItem
				area="true-positive"
				positive
				title="True Positives"
				total={total}
				value={props.truePositives}
			/>
			<ConfusionMatrixItem
				area="false-positive"
				title="False Positives"
				total={total}
				value={props.falsePositives}
			/>
			<ConfusionMatrixItem
				area="false-negative"
				title="False Negatives"
				total={total}
				value={props.falseNegatives}
			/>
			<ConfusionMatrixItem
				area="true-negative"
				positive
				title="True Negatives"
				total={total}
				value={props.trueNegatives}
			/>
		</div>
	)
}

type ConfusionMatrixItemProps = {
	area: string
	positive?: boolean
	title: string
	total: number | null
	value: number | null
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

let positiveItemWrapperClass = cssClass()
let positiveItemWrapperCss = css({
	[`.${positiveItemWrapperClass}`]: {
		backgroundColor: config.trueBackgroundColor,
		color: config.trueForegroundColor,
	},
})
let negativeItemWrapperClass = cssClass()
let negativeItemWrapperCss = css({
	[`.${negativeItemWrapperClass}`]: {
		backgroundColor: config.falseBackgroundColor,
		color: config.falseForegroundColor,
	},
})

let titleClass = cssClass()
let titleCss = css({
	[desktop]: {
		fontSize: '1.25rem',
	},
	[mobile]: {
		fontSize: '1rem',
	},
})

let valueClass = cssClass()
let valueCss = css({
	[desktop]: {
		fontSize: '2rem',
	},
	[mobile]: {
		fontSize: '1.5rem',
	},
})

let percentClass = cssClass()
let percentCss = css({
	[desktop]: {
		fontSize: '1.25rem',
	},
	[mobile]: {
		fontSize: '1rem',
	},
})

function ConfusionMatrixItem(props: ConfusionMatrixItemProps) {
	useCss(
		itemWrapperCss,
		titleCss,
		valueCss,
		percentCss,
		props.positive ? positiveItemWrapperCss : negativeItemWrapperCss,
	)
	let itemWrapperStyle = {
		gridArea: props.area,
	}
	return (
		<div
			class={cx(
				itemWrapperClass,
				props.positive ? positiveItemWrapperClass : negativeItemWrapperClass,
			)}
			style={itemWrapperStyle}
		>
			<div class={titleClass}>{props.title}</div>
			<div class={valueClass}>{defaultValueFormatter(props.value)}</div>
			<div class={percentClass}>
				{props.value === null || props.total === null
					? 'N/A'
					: formatPercent(props.value / props.total, 2)}
			</div>
		</div>
	)
}

function defaultValueFormatter(value: number | null) {
	return value === null ? 'N/A' : value.toString()
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
		gridGap: '0.25rem',
		justifyItems: 'center',
	},
})

function ConfusionMatrixLabel(props: ConfusionMatrixLabelProps) {
	useCss(labelCss)
	let style = {
		gridArea: props.area,
		justifyItems: props.left ? 'end' : 'auto',
	}
	return (
		<div class={labelClass} style={style}>
			{props.children}
		</div>
	)
}
