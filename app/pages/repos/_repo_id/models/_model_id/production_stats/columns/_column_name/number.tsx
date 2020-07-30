import { MetricsRow } from 'common/metrics_row'
import {
	DateWindow,
	DateWindowInterval,
	intervalChartTitle,
	overallChartTitle,
} from 'common/time_charts'
import {
	productionColor,
	productionTextColor,
	trainingColor,
	trainingTextColor,
} from 'common/tokens'
import { h, ui } from 'deps'

export type Props = {
	alert: string | null
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	intervals: Array<{
		label: string
		stats: {
			max: number
			mean: number
			min: number
			p25: number
			p50: number
			p75: number
		} | null
	}>
	name: string
	overall: {
		absentCount: number
		invalidCount: number
		label: string
		rowCount: number
		stats: {
			production: {
				max: number
				mean: number
				min: number
				p25: number
				p50: number
				p75: number
				std: number
			} | null
			training: {
				max: number
				mean: number
				min: number
				p25: number
				p50: number
				p75: number
				std: number
			}
		}
	}
}

export function Number(props: Props) {
	let intervalChartData = [
		{
			color: productionColor,
			data: props.intervals.map((entry, index) => ({
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
			title: `Production Stats for "${props.name}"`,
		},
	]

	let overallChartData: ui.BoxChartSeries[] = [
		{
			color: trainingColor,
			data: props.overall.stats
				? [
						{
							x: 0,
							y: {
								max: props.overall.stats.training.max,
								min: props.overall.stats.training.min,
								p25: props.overall.stats.training.p25,
								p50: props.overall.stats.training.p50,
								p75: props.overall.stats.training.p75,
							},
						},
				  ]
				: [],
			title: `Training Stats for "${props.name}"`,
		},
		{
			color: productionColor,
			data: props.overall.stats.production
				? [
						{
							x: 0,
							y: {
								max: props.overall.stats.production.max,
								min: props.overall.stats.production.min,
								p25: props.overall.stats.production.p25,
								p50: props.overall.stats.production.p50,
								p75: props.overall.stats.production.p75,
							},
						},
				  ]
				: [],
			title: `Production Stats for "${props.name}"`,
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
				<ui.Alert level={ui.Level.Warning}>{props.alert}</ui.Alert>
			)}
			<ui.Card>
				<ui.BoxChart
					data={overallChartData}
					title={statsOverallChartTitle}
					xAxisLabelFormatter={_ => props.overall.label}
				/>
			</ui.Card>
			<ui.Card>
				<ui.BoxChart
					data={intervalChartData}
					title={statsIntervalChartTitle}
					xAxisLabelFormatter={i => props.intervals[i].label}
				/>
			</ui.Card>
			<MetricsRow>
				<ui.Card>
					<ui.NumberChart
						title="Row Count"
						value={props.overall.rowCount.toString()}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberChart
						title="Absent Count"
						value={props.overall.absentCount.toString()}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberChart
						title="Invalid Count"
						value={props.overall.invalidCount.toString()}
					/>
				</ui.Card>
			</MetricsRow>
			<MetricsRow>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						textColorA={trainingTextColor}
						textColorB={productionTextColor}
						title="Min"
						valueA={props.overall.stats.training.min}
						valueATitle="Training"
						valueB={props.overall.stats.production?.min ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						textColorA={trainingTextColor}
						textColorB={productionTextColor}
						title="Max"
						valueA={props.overall.stats.training.max}
						valueATitle="Training"
						valueB={props.overall.stats.production?.max ?? null}
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
						textColorA={trainingTextColor}
						textColorB={productionTextColor}
						title="Mean"
						valueA={props.overall.stats.training.mean}
						valueATitle="Training"
						valueB={props.overall.stats.production?.mean ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={trainingColor}
						colorB={productionColor}
						textColorA={trainingTextColor}
						textColorB={productionTextColor}
						title="Standard Deviation"
						valueA={props.overall.stats.training.std}
						valueATitle="Training"
						valueB={props.overall.stats.production?.std ?? null}
						valueBTitle="Production"
						valueFormatter={value => ui.formatNumber(value)}
					/>
				</ui.Card>
			</MetricsRow>
		</ui.S2>
	)
}
