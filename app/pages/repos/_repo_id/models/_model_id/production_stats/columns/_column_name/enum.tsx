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
	absentCount: number
	alert: string | null
	columnName: string
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	intervalChartData: Array<{
		histogram: Array<[string, number]>
		label: string
	}>
	invalidCount: number
	overallChartData: Array<
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
	overallInvalidChartData: Array<[string, number]> | null
	rowCount: number
}

export function Enum(props: Props) {
	let overallChartData = [
		{
			color: trainingColor,
			data: props.overallChartData.map(([label, entry], i) => ({
				label,
				x: i,
				y: entry.trainingFraction,
			})),
			title: 'Training',
		},
		{
			color: productionColor,
			data: props.overallChartData.map(([label, entry], i) => ({
				label,
				x: i,
				y: entry.productionFraction,
			})),
			title: 'Production',
		},
	]
	let overallDistributionChartTitle = overallChartTitle(
		props.dateWindow,
		`Distribution of Unique Values for ${props.columnName}`,
	)

	let categories = props.intervalChartData[0].histogram.map(x => x[0])
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
	let intervalChartData = ui.times(
		props.intervalChartData[0].histogram.length,
		i => ({
			color: colorOptions[i % colorOptions.length],
			data: props.intervalChartData.map((entry, j) => ({
				label: entry.label,
				x: j,
				y: entry.histogram !== null ? entry.histogram[i][1] : null,
			})),
			title: categories[i],
		}),
	)

	let intervalDistributionChartTitle = intervalChartTitle(
		props.dateWindowInterval,
		`Distribution of Unique Values for ${props.columnName}`,
	)

	return (
		<ui.S2>
			{props.alert && (
				<ui.Alert level={ui.Level.Danger}>{props.alert}</ui.Alert>
			)}
			<ui.Card>
				<ui.BarChart
					data={overallChartData}
					id="enum_overall"
					title={overallDistributionChartTitle}
					xAxisTitle={props.columnName}
					yAxisTitle="Percent"
					yMax={1}
				/>
			</ui.Card>
			<ui.Card>
				<ui.BarChart
					data={intervalChartData}
					id="enum_intervals"
					title={intervalDistributionChartTitle}
					yAxisTitle="Count"
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
					{props.overallChartData.map(([value, entry]) => (
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
			{props.overallInvalidChartData && (
				<>
					<ui.H2>{'Invalid Values'}</ui.H2>
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>{'Value'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Count'}</ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.overallInvalidChartData.map(([value, count], i) => (
								<ui.TableRow key={i}>
									<ui.TableCell>{value}</ui.TableCell>
									<ui.TableCell>{ui.formatNumber(count)}</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				</>
			)}
		</ui.S2>
	)
}
