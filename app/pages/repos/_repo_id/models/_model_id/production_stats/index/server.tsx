import {
	ColumnType,
	MulticlassClassificationChartEntry,
	Props,
	RegressionChartEntry,
	Task,
} from './props'
import { BarChart, BoxChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { DateWindowSelectField } from 'common/date_window_select_field'
import { renderPage } from 'common/render'
import {
	DateWindow,
	DateWindowInterval,
	intervalChartTitle,
	overallChartTitle,
} from 'common/time'
import {
	EnumColumnToken,
	NumberColumnToken,
	TextColumnToken,
} from 'common/tokens'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default function ProductionStatsIndexPage(props: Props) {
	let predictionCountData = [
		{
			color: ui.colors.blue,
			data: props.predictionCountChart.map((entry, i) => ({
				label: entry.label,
				x: i,
				y: entry.count,
			})),
			title: 'Prediction Count',
		},
	]
	let predictionCountTitle = intervalChartTitle(
		props.dateWindowInterval,
		'Prediction Count',
	)
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.ProductionStats}
		>
			<ui.S1>
				<ui.H1>{'Production Stats'}</ui.H1>
				<ui.Form>
					<DateWindowSelectField dateWindow={props.dateWindow} />
					<noscript>
						<ui.Button>{'Submit'}</ui.Button>
					</noscript>
				</ui.Form>
				{props.predictionStatsIntervalChart.type === Task.Regression ? (
					<ui.Card>
						<RegressionProductionStatsIntervalChart
							chartData={props.predictionStatsIntervalChart.data}
							dateWindow={props.dateWindow}
							dateWindowInterval={props.dateWindowInterval}
						/>
					</ui.Card>
				) : (
					<ui.Card>
						<MulticlassClassificationProductionStatsIntervalChart
							chartData={props.predictionStatsIntervalChart.data}
							dateWindow={props.dateWindow}
							dateWindowInterval={props.dateWindowInterval}
						/>
					</ui.Card>
				)}
				<ui.Card>
					<BarChart
						data={predictionCountData}
						id="prediction_count"
						title={predictionCountTitle}
					/>
				</ui.Card>
				{props.predictionStatsChart.type === Task.Regression ? (
					<ui.Card>
						<RegressionProductionStatsChart
							chartData={props.predictionStatsChart.data}
							dateWindow={props.dateWindow}
						/>
					</ui.Card>
				) : (
					<ui.Card>
						<MulticlassClassificationProductionStatsChart
							chartData={props.predictionStatsChart.data}
							dateWindow={props.dateWindow}
						/>
					</ui.Card>
				)}
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableRow>
							<ui.TableHeaderCell>{'Status'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Column'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Type'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Absent Count'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Invalid Count'}</ui.TableHeaderCell>
						</ui.TableRow>
					</ui.TableHeader>
					<ui.TableBody>
						{props.overallColumnStatsTable.map(column => (
							<ui.TableRow key={column.name}>
								<ui.TableCell>
									{column.alert ? (
										<ui.AlertIcon alert={column.alert} level={ui.Level.Danger}>
											{'!'}
										</ui.AlertIcon>
									) : (
										<ui.AlertIcon alert="All good" level={ui.Level.Success}>
											{'✓'}
										</ui.AlertIcon>
									)}
								</ui.TableCell>
								<ui.TableCell>
									<ui.Link href={`./columns/${column.name}`}>
										{column.name}
									</ui.Link>
								</ui.TableCell>
								<ui.TableCell>
									{column.columnType === ColumnType.Number ? (
										<NumberColumnToken />
									) : column.columnType === ColumnType.Enum ? (
										<EnumColumnToken />
									) : column.columnType === ColumnType.Text ? (
										<TextColumnToken />
									) : null}
								</ui.TableCell>
								<ui.TableCell>{column.absentCount}</ui.TableCell>
								<ui.TableCell>{column.invalidCount}</ui.TableCell>
							</ui.TableRow>
						))}
					</ui.TableBody>
				</ui.Table>
			</ui.S1>
		</ModelLayout>,
	)
}

function RegressionProductionStatsChart(props: {
	chartData: RegressionChartEntry
	dateWindow: DateWindow
}) {
	let data = [
		{
			color: ui.colors.green,
			data: [
				{
					label: props.chartData.label,
					x: 0,
					y: {
						max: props.chartData.quantiles.training.max,
						min: props.chartData.quantiles.training.min,
						p25: props.chartData.quantiles.training.p25,
						p50: props.chartData.quantiles.training.p50,
						p75: props.chartData.quantiles.training.p75,
					},
				},
			],
			title: 'training quantiles',
		},
		{
			color: ui.colors.blue,
			data: [
				{
					label: props.chartData.label,
					x: 0,
					y:
						props.chartData.quantiles.production !== null
							? {
									max: props.chartData.quantiles.production.max,
									min: props.chartData.quantiles.production.min,
									p25: props.chartData.quantiles.production.p25,
									p50: props.chartData.quantiles.production.p50,
									p75: props.chartData.quantiles.production.p75,
							  }
							: null,
				},
			],
			title: 'production quantiles',
		},
	]
	let title = overallChartTitle(
		props.dateWindow,
		'Prediction Distribution Stats',
	)
	return <BoxChart data={data} id="quantiles_overall" title={title} />
}

function RegressionProductionStatsIntervalChart(props: {
	chartData: RegressionChartEntry[]
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
}) {
	let data = [
		{
			color: ui.colors.blue,
			data: props.chartData.map((entry, i) => ({
				label: entry.label,
				x: i,
				y:
					entry.quantiles.production !== null
						? {
								max: entry.quantiles.production.max,
								min: entry.quantiles.production.min,
								p25: entry.quantiles.production.p25,
								p50: entry.quantiles.production.p50,
								p75: entry.quantiles.production.p75,
						  }
						: null,
			})),
			title: 'quantiles',
		},
	]
	let title = intervalChartTitle(
		props.dateWindowInterval,
		'Prediction Distribution Stats',
	)
	return <BoxChart data={data} id="quantiles_intervals" title={title} />
}

function MulticlassClassificationProductionStatsChart(props: {
	chartData: MulticlassClassificationChartEntry
	dateWindow: DateWindow
}) {
	let options = props.chartData.histogram.production.map(x => x[0])
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
	let data = ui.times(props.chartData.histogram.production.length, i => ({
		color: colorOptions[i % colorOptions.length],
		data:
			props.chartData.histogram !== null
				? [
						{
							label: props.chartData.label,
							x: 0,
							y: props.chartData.histogram.production[i][1],
						},
				  ]
				: [],
		title: options[i],
	}))

	let title = overallChartTitle(props.dateWindow, 'Prediction Stats')
	return <BarChart data={data} id="histogram_overall" title={title} />
}

function MulticlassClassificationProductionStatsIntervalChart(props: {
	chartData: MulticlassClassificationChartEntry[]
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
}) {
	let options = props.chartData[0].histogram.production.map(x => x[0])
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
	let data = ui.times(props.chartData[0].histogram.production.length, i => ({
		color: colorOptions[i % colorOptions.length],
		data: props.chartData.map((entry, j) => ({
			label: entry.label,
			x: j,
			y: entry?.histogram.production[i][1] ?? null,
		})),
		title: options[i],
	}))
	let title = intervalChartTitle(props.dateWindowInterval, 'Prediction Stats')
	return <BarChart data={data} id="histogram_intervals" title={title} />
}
