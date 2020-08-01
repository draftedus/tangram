import { Children, css, cx, h, useCss } from '../deps'
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

let wrapperCss = css({
	[`.confusion-matrix-wrapper`]: {
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
		<div class="confusion-matrix-wrapper">
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

let itemWrapperCss = css({
	[`.confusion-matrix-item-wrapper`]: {
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

let positiveItemWrapperCss = css({
	[`.confusion-matrix-item-positive-wrapper`]: {
		backgroundColor: config.trueBackgroundColor,
		color: config.trueForegroundColor,
	},
})
let negativeItemWrapperCss = css({
	[`.confusion-matrix-item-negative-wrapper`]: {
		backgroundColor: config.falseBackgroundColor,
		color: config.falseForegroundColor,
	},
})

let titleCss = css({
	[desktop]: {
		[`.confusion-matrix-item-title`]: {
			fontSize: '1.25rem',
		},
	},
	[mobile]: {
		[`.confusion-matrix-item-title`]: {
			fontSize: '1rem',
		},
	},
})

let valueCss = css({
	[desktop]: {
		[`.confusion-matrix-item-value`]: {
			fontSize: '2rem',
		},
	},
	[mobile]: {
		[`.confusion-matrix-item-value`]: {
			fontSize: '1.5rem',
		},
	},
})

let percentCss = css({
	[desktop]: {
		[`.confusion-matrix-item-percent`]: {
			fontSize: '1.25rem',
		},
	},
	[mobile]: {
		[`.confusion-matrix-item-percent`]: {
			fontSize: '1rem',
		},
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
				'confusion-matrix-item-wrapper',
				props.positive
					? 'confusion-matrix-item-positive-wrapper'
					: 'confusion-matrix-item-negative-wrapper',
			)}
			style={itemWrapperStyle}
		>
			<div class="confusion-matrix-item-title">{props.title}</div>
			<div class="confusion-matrix-item-value">
				{defaultValueFormatter(props.value)}
			</div>
			<div class="confusion-matrix-item-percent">
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

let labelCss = css({
	[`.confusion-matrix-label`]: {
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
		<div class="confusion-matrix-label" style={style}>
			{props.children}
		</div>
	)
}
