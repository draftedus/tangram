import { Children, cx, h } from '../deps'
import { variables } from '../theme'
import { Token } from '../token'

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

export function ConfusionMatrixComparison(
	props: ConfusionMatrixComparisonProps,
) {
	return (
		<div class="confusion-matrix-comparison-wrapper">
			<ConfusionMatrixLabel area="actual-true-label">
				<div>{'Actual'}</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="actual-false-label">
				<div>{'Actual Not'}</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-true-label" left={true}>
				<div>{'Predicted'}</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-false-label" left={true}>
				<div>{'Predicted Not'}</div>
				<Token color={variables.colors.accent}>{props.classLabel}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixComparisonItem
				area="true-positive"
				colorA={props.colorA}
				colorB={props.colorB}
				label="True Positives"
				textColorA={props.textColorA}
				textColorB={props.textColorB}
				true={true}
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
				true={true}
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

function ConfusionMatrixComparisonItem(props: ConfusionMatrixItemProps) {
	let valueFormatter = props.valueFormatter || (v => v.toString())
	let wrapperStyle = {
		gridArea: props.area,
	}
	return (
		<div
			class={cx(
				'confusion-matrix-comparison-item-wrapper',
				props.true
					? 'confusion-matrix-comparison-item-positive-wrapper'
					: 'confusion-matrix-comparison-item-negative-wrapper',
			)}
			style={wrapperStyle}
		>
			<div class="confusion-matrix-comparison-item-title">{props.label}</div>
			<div class=".confusion-matrix-comparison-number-comparison-wrapper">
				<div class="confusion-matrix-comparison-item-value">
					{props.valueA === null ? 'N/A' : valueFormatter(props.valueA)}
				</div>
				<div class="confusion-matrix-comparison-item-value">
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

function ConfusionMatrixLabel(props: ConfusionMatrixLabelProps) {
	let style = {
		gridArea: props.area,
		justifyItems: props.left ? 'end' : 'center',
	}
	return (
		<div class="confusion-matrix-comparison-label" style={style}>
			{props.children}
		</div>
	)
}
