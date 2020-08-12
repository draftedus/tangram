import { MetricsRow } from 'common/metrics_row'
import {
	DateWindow,
	DateWindowInterval,
	intervalChartTitle,
	overallChartTitle,
} from 'common/time_charts'
import { productionColor, trainingColor } from 'common/tokens'
import { Fragment, h, ui } from 'deps'

export type Props = {
	alert: string | null
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	intervals: Array<{
		histogram: Array<[string, number]>
		label: string
	}>
	name: string
	overall: {
		absentCount: number
		histogram: Array<
			[
				string,
				{
					productionCount: number
					productionFraction: number
					trainingCount: number
					trainingFraction: number
				},
			]
		>
		invalidCount: number
		invalidHistogram: Array<[string, number]> | null
		label: string
		rowCount: number
	}
}

export function Enum(props: Props) {
	let overallChartData = [
		{
			color: trainingColor,
			data: props.overall.histogram.map(([_value, entry], i) => ({
				x: i,
				y: entry.trainingFraction,
			})),
			title: 'Training',
		},
		{
			color: productionColor,
			data: props.overall.histogram.map(([_value, entry], i) => ({
				x: i,
				y: entry.productionFraction,
			})),
			title: 'Production',
		},
	]
	let overallDistributionChartTitle = overallChartTitle(
		props.dateWindow,
		`Distribution of Unique Values for ${props.name}`,
	)

	let categories = props.intervals[0].histogram.map(x => x[0])
	let colorOptions = [
		ui.colors.green,
		ui.colors.blue,
		ui.colors.indigo,
		ui.colors.purple,
		ui.colors.pink,
		ui.colors.red,
		ui.colors.orange,
		ui.colors.yellow,
	]
	let intervalChartData = ui.times(props.intervals[0].histogram.length, i => ({
		color: colorOptions[i % colorOptions.length],
		data: props.intervals.map((entry, j) => ({
			x: j,
			y: entry.histogram !== null ? entry.histogram[i][1] : null,
		})),
		title: categories[i],
	}))

	let intervalDistributionChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		`Distribution of Unique Values for ${props.name}`,
	)

	return (
		<ui.S2>
			{props.alert && (
				<ui.Alert level={ui.Level.Warning}>{props.alert}</ui.Alert>
			)}
			<ui.Card>
				<ui.BarChart
					data={overallChartData}
					id="enum_overall"
					title={overallDistributionChartTitle}
					xAxisLabelFormatter={x => props.overall.histogram[x][0]}
					xAxisTitle={props.name}
					yAxisLabelFormatter={value => ui.formatPercent(value, 2)}
					yAxisTitle="Percent"
					yMax={1}
				/>
			</ui.Card>
			<ui.Card>
				<ui.BarChart
					data={intervalChartData}
					id="enum_intervals"
					title={intervalDistributionChartTitle}
					xAxisLabelFormatter={i => props.intervals[i].label}
					yAxisTitle="Count"
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
			<ui.H2>{'Unique Values'}</ui.H2>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Value'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Training Count'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Production Count'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Training Fraction'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Production Fraction'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{props.overall.histogram.map(([value, entry]) => (
						<ui.TableRow key={value}>
							<ui.TableCell>{value}</ui.TableCell>
							<ui.TableCell>
								{ui.formatNumber(entry.trainingCount)}
							</ui.TableCell>
							<ui.TableCell>
								{ui.formatNumber(entry.productionCount)}
							</ui.TableCell>
							<ui.TableCell>
								{ui.formatPercent(entry.trainingFraction, 2)}
							</ui.TableCell>
							<ui.TableCell>
								{ui.formatPercent(entry.productionFraction, 2)}
							</ui.TableCell>
						</ui.TableRow>
					))}
				</ui.TableBody>
			</ui.Table>
			{props.overall.invalidHistogram && (
				<Fragment>
					<ui.H2>{'Invalid Values'}</ui.H2>
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>{'Value'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Count'}</ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.overall.invalidHistogram.map(([value, count], i) => (
								<ui.TableRow key={i}>
									<ui.TableCell>{value}</ui.TableCell>
									<ui.TableCell>{ui.formatNumber(count)}</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				</Fragment>
			)}
		</ui.S2>
	)
}
