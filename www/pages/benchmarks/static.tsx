import { BarChart } from '@tangramhq/charts'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { PageLayout } from 'layouts/page_layout'
import { h } from 'preact'

type Props = {
	pinwheelInfo: PinwheelInfo
}

type BenchmarkDatasets = { [key: string]: BenchmarkLibraries }
type BenchmarkLibraries = { [key: string]: number }

export default function Home(props: Props) {
	return renderPage(
		<PageLayout background={true} pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Benchmarks'}</ui.H1>
				<ui.P>
					{
						'Under the hood, the Tangram CLI uses Tangram Tree and Tangram Linear to train machine learning models.'
					}
				</ui.P>
				<ui.S2>
					<ui.H2>{'Tangram Tree Memory Usage'}</ui.H2>
					<TreeMemoryBenchmark />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Tangram Tree Training Time'}</ui.H2>
					<TreeTimeBenchmark />
				</ui.S2>
			</ui.S1>
		</PageLayout>,
	)
}

function TreeMemoryBenchmark() {
	let data: BenchmarkDatasets = {
		allstate: {
			lightgbm: 12.372716,
			tangram: 4.831324,
			xgboost: 0,
		},
		flights: {
			lightgbm: 2.007908,
			tangram: 1.140732,
			xgboost: 0,
		},
		higgs: {
			lightgbm: 11.618204,
			tangram: 2.455276,
			xgboost: 12.743936,
		},
	}
	let barChartMemoryData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.tangram },
				{ label: 'flights', x: 1, y: data.flights.tangram },
				{ label: 'allstate', x: 2, y: data.allstate.tangram },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.lightgbm },
				{ label: 'flights', x: 1, y: data.flights.lightgbm },
				{ label: 'allstate', x: 2, y: data.allstate.lightgbm },
			],
			title: 'lightGBM',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.xgboost },
				{ label: 'flights', x: 1, y: null },
				{ label: 'allstate', x: 2, y: null },
			],
			title: 'xGBoost',
		},
	]
	let formatGB = (value: number) => `${ui.formatNumber(value, 4)} GB`
	let formatGBDiff = (value: number) => `${ui.formatNumber(value, 4)}x`
	return (
		<div className="benchmarks_table_chart_grid">
			<ui.Table>
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Dataset'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Library'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Memory (GB)'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'v. Tangram'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{['higgs', 'flights', 'allstate'].map(dataset =>
						['tangram', 'lightgbm', 'xgboost'].map(library => (
							<ui.TableRow key={dataset + library}>
								<ui.TableCell>{dataset}</ui.TableCell>
								<ui.TableCell>{library}</ui.TableCell>
								<ui.TableCell>{formatGB(data[dataset][library])}</ui.TableCell>
								<ui.TableCell>
									{formatGBDiff(
										data[dataset][library] / data[dataset]['tangram'],
									)}
								</ui.TableCell>
							</ui.TableRow>
						)),
					)}
				</ui.TableBody>
			</ui.Table>
			<BarChart
				data={barChartMemoryData}
				groupGap={10}
				id="memory_benchmark"
				title="Memory Usage Benchmark (GB)"
				xAxisTitle="Library"
				yAxisTitle="Memory Usage (GB)"
				yMin={0}
			/>
		</div>
	)
}

function TreeTimeBenchmark() {
	let data: BenchmarkDatasets = {
		allstate: {
			lightgbm: 69.314861553,
			tangram: 62.931978142,
			xgboost: 0,
		},
		flights: {
			lightgbm: 45.382501249,
			tangram: 37.496518463,
			xgboost: 0,
		},
		higgs: {
			lightgbm: 124.955734558,
			tangram: 100.176465602,
			xgboost: 101.76733798,
		},
	}
	let barChartTimeData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.tangram },
				{ label: 'flights', x: 1, y: data.flights.tangram },
				{ label: 'allstate', x: 2, y: data.allstate.tangram },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.lightgbm },
				{ label: 'flights', x: 1, y: data.flights.lightgbm },
				{ label: 'allstate', x: 2, y: data.allstate.lightgbm },
			],
			title: 'lightGBM',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.xgboost },
				{ label: 'flights', x: 1, y: null },
				{ label: 'allstate', x: 2, y: null },
			],
			title: 'XGboost',
		},
	]
	let formatTime = (value: number) => `${ui.formatNumber(value, 4)} seconds`
	let formatTimeDiff = (value: number) => `${ui.formatNumber(value, 4)}x`
	return (
		<div className="benchmarks_table_chart_grid">
			<ui.Table>
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Dataset'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Library'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Memory (GB)'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'v. Tangram'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{['higgs', 'flights', 'allstate'].map(dataset =>
						['tangram', 'lightgbm', 'xgboost'].map(library => (
							<ui.TableRow key={dataset + library}>
								<ui.TableCell>{dataset}</ui.TableCell>
								<ui.TableCell>{library}</ui.TableCell>
								<ui.TableCell>
									{formatTime(data[dataset][library])}
								</ui.TableCell>
								<ui.TableCell>
									{formatTimeDiff(
										data[dataset][library] / data[dataset]['tangram'],
									)}
								</ui.TableCell>
							</ui.TableRow>
						)),
					)}
				</ui.TableBody>
			</ui.Table>
			<BarChart
				data={barChartTimeData}
				groupGap={30}
				id="time_benchmark"
				title="Training Time Benchmark (seconds)"
				xAxisTitle="Library"
				yAxisTitle="Training Time (seconds)"
			/>
		</div>
	)
}

function TreeAUCBenchmark() {
	let data = {
		flights: {
			lightgbm: 0.7807312,
			tangram: 0.7815357,
			xgboost: 0,
		},
		higgs: {
			lightgbm: 0.83145106,
			tangram: 0.8320089,
			xgboost: 0.81292254,
		},
	}
	let barChartAUCData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.tangram },
				{ label: 'flights', x: 1, y: data.flights.tangram },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.lightgbm },
				{ label: 'flights', x: 1, y: data.flights.lightgbm },
			],
			title: 'lightGBM',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.xgboost },
				{ label: 'flights', x: 1, y: data.flights.xgboost },
			],
			title: 'XGboost',
		},
	]

	return (
		<div className="benchmarks_table_chart_grid">
			<ui.Table>
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Dataset'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Library'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Memory (GB)'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'v. Tangram'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{['higgs', 'flights'].map(dataset =>
						['tangram', 'lightgbm', 'xgboost'].map(library => (
							<ui.TableRow key={dataset + library}>
								<ui.TableCell>{dataset}</ui.TableCell>
								<ui.TableCell>{library}</ui.TableCell>
								<ui.TableCell>{formatAUC(data[dataset][library])}</ui.TableCell>
							</ui.TableRow>
						)),
					)}
				</ui.TableBody>
			</ui.Table>
			<BarChart
				data={barChartAUCData}
				groupGap={30}
				id="auc_benchmark"
				title="AUC Benchmark"
				xAxisTitle="Library"
				yAxisTitle="AUC"
				yMax={1}
				yMin={0}
			/>
		</div>
	)
}
