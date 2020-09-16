import { Token } from './token'
import { formatPercent } from './util'
import { cx } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

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

export function ConfusionMatrix(props: ConfusionMatrixProps) {
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
				<div>{'Actual'}</div>
				<Token>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="actual-false-label">
				<div>{'Actual Not'}</div>
				<Token>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-true-label" left={true}>
				<div>{'Predicted'}</div>
				<Token>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-false-label" left={true}>
				<div>{'Predicted Not'}</div>
				<Token>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixItem
				area="true-positive"
				positive={true}
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
				positive={true}
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

function ConfusionMatrixItem(props: ConfusionMatrixItemProps) {
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
	children?: ComponentChildren
	left?: boolean
}

function ConfusionMatrixLabel(props: ConfusionMatrixLabelProps) {
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
