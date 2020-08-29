import { MetricsRow } from 'common/metrics_row'
import {
	DateWindow,
	DateWindowInterval,
	intervalChartTitle,
	overallChartTitle,
} from 'common/time_charts'
import { productionColor, trainingColor } from 'common/tokens'
import { h, ui } from 'deps'

export type Props = {
	absentCount: number
	alert: string | null
	columnName: string
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	intervalBoxChartData: Array<{
		label: string
		stats: {
			max: number
			min: number
			p25: number
			p50: number
			p75: number
		} | null
	}>
	invalidCount: number
	maxComparison: {
		production: number | null
		training: number
	}
	meanComparison: {
		production: number | null
		training: number
	}
	minComparison: {
		production: number | null
		training: number
	}
	overallBoxChartData: {
		production: {
			max: number
			min: number
			p25: number
			p50: number
			p75: number
		} | null
		training: {
			max: number
			min: number
			p25: number
			p50: number
			p75: number
		}
	}
	rowCount: number
	stdComparison: {
		production: number | null
		training: number
	}
}

export function Number(props: Props) {
	let intervalBoxChartData = [
		{
			color: productionColor,
			data: props.intervalBoxChartData.map((entry, index) => ({
				label: entry.label,
				x: index,
				y: entry.stats
					? {
							max: entry.stats.max,
							min: entry.stats.min,
							p25: entry.stats.p25,
							p50: entry.stats.p50,
							p75: entry.stats.p75,
					  }
					: null,
			})),
			title: `Production Stats for "${props.columnName}"`,
		},
	]
	let overallBoxChartData = [
		{
			color: trainingColor,
			data: [
				{
					label: 'Training',
					x: 0,
					y: {
						max: props.overallBoxChartData.training.max,
						min: props.overallBoxChartData.training.min,
						p25: props.overallBoxChartData.training.p25,
						p50: props.overallBoxChartData.training.p50,
						p75: props.overallBoxChartData.training.p75,
					},
				},
			],
			title: `Training Stats for "${props.columnName}"`,
		},
		{
			color: productionColor,
			data: props.overallBoxChartData.production
				? [
						{
							label: 'Production',
							x: 0,
							y: {
								max: props.overallBoxChartData.production.max,
								min: props.overallBoxChartData.production.min,
								p25: props.overallBoxChartData.production.p25,
								p50: props.overallBoxChartData.production.p50,
								p75: props.overallBoxChartData.production.p75,
							},
						},
				  ]
				: [],
			title: `Production Stats for "${props.columnName}"`,
		},
	]
	let statsOverallChartTitle = overallChartTitle(props.dateWindow, 'Stats')
	let statsIntervalChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		'Stats',
	)
	return (
		<ui.S2>
			{props.alert && (
				<ui.Alert level={ui.Level.Danger}>{props.alert}</ui.Alert>
			)}
			<ui.Card>
				<ui.BoxChart
					data={overallBoxChartData}
					id="number_overall"
					title={statsOverallChartTitle}
				/>
			</ui.Card>
			<ui.Card>
				<ui.BoxChart
					data={intervalBoxChartData}
					id="number_intervals"
					title={statsIntervalChartTitle}
				/>
			</ui.Card>
			<MetricsRow>
				<ui.Card>
					<ui.NumberChart title="Row Count" value={props.rowCount.toString()} />
				</ui.Card>
				<ui.Card>
					<ui.NumberChart
						title="Absent Count"
						value={props.absentCount.toString()}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberChart
						title="Invalid Count"
						value={props.invalidCount.toString()}
					/>
				</ui.Card>
			</MetricsRow>
			<MetricsRow>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						title="Min"
						valueA={props.minComparison.training}
						valueATitle="Training"
						valueB={props.minComparison.production ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						title="Max"
						valueA={props.maxComparison.training}
						valueATitle="Training"
						valueB={props.maxComparison.production ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
			</MetricsRow>
			<MetricsRow>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						title="Mean"
						valueA={props.meanComparison.training}
						valueATitle="Training"
						valueB={props.meanComparison.production ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						title="Standard Deviation"
						valueA={props.stdComparison.training}
						valueATitle="Training"
						valueB={props.stdComparison.production ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
			</MetricsRow>
		</ui.S2>
	)
}
