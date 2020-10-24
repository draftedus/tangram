import { BarChart } from '@tangramhq/charts'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { PageLayout } from 'layouts/page_layout'
import { h } from 'preact'

type BenchmarkDatasets = { [key: string]: BenchmarkLibraries }
type BenchmarkLibraries = {
	[key: string]: BenchmarkResults
}
type BenchmarkResults = { [key: string]: number }

let data: BenchmarkDatasets = {
	allstate: {
		lightgbm: { duration: 73.958655182, memory: 12.281084, mse: 1587.0221 },
		sklearn: { duration: 72.647664062, memory: 10.928152, mse: 1583.6423 },
		tangram: { duration: 60.637947846, memory: 4.832316, mse: 1587.8885 },
		xgboost: { duration: 77.592920662, memory: 12.41334, mse: 1581.0436 },
	},
	flights: {
		lightgbm: { auc_roc: 0.7807312, duration: 42.959502341, memory: 1.984692 },
		sklearn: { auc_roc: 0.75876635, duration: 58.51804273, memory: 2.542824 },
		tangram: { auc_roc: 0.7815357, duration: 37.162440456, memory: 1.140836 },
		xgboost: { auc_roc: 0.75779957, duration: 47.069649134, memory: 2.420956 },
	},
	higgs: {
		lightgbm: {
			auc_roc: 0.83145106,
			duration: 121.337377524,
			memory: 11.616628,
		},
		sklearn: { auc_roc: 0.831599, duration: 205.640246711, memory: 9.294468 },
		tangram: { auc_roc: 0.8320089, duration: 98.863112924, memory: 2.45728 },
		xgboost: {
			auc_roc: 0.81292254,
			duration: 105.822569973,
			memory: 12.734708,
		},
	},
}

export default (pageInfo: PageInfo) => {
	return renderPage(
		<PageLayout
			background={true}
			clientJsSrc={pageInfo.clientJsSrc}
			cssSrcs={pageInfo.cssSrcs}
			preloadJsSrcs={pageInfo.preloadJsSrcs}
		>
			<ui.S1>
				<ui.H1>{'Benchmarks'}</ui.H1>
				<ui.P>
					{
						'Under the hood, the Tangram CLI uses Tangram Tree and Tangram Linear to train machine learning models.'
					}
				</ui.P>
				<ui.S2>
					<ui.H2>{'Memory Usage'}</ui.H2>
					<TreeMemoryBenchmark />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Training Time'}</ui.H2>
					<TreeTimeBenchmark />
				</ui.S2>
				<ui.S2>
					<ui.H2>{'Area Under the Receiver Operating Characteristic'}</ui.H2>
					<TreeAUCBenchmark />
				</ui.S2>
			</ui.S1>
		</PageLayout>,
	)
}

function TreeMemoryBenchmark() {
	let barChartMemoryData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.tangram.memory },
				{ label: 'flights', x: 1, y: data.flights.tangram.memory },
				{ label: 'allstate', x: 2, y: data.allstate.tangram.memory },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.lightgbm.memory },
				{ label: 'flights', x: 1, y: data.flights.lightgbm.memory },
				{ label: 'allstate', x: 2, y: data.allstate.lightgbm.memory },
			],
			title: 'lightgbm',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.xgboost.memory },
				{ label: 'flights', x: 1, y: data.flights.xgboost.memory },
				{ label: 'allstate', x: 2, y: data.allstate.xgboost.memory },
			],
			title: 'xgboost',
		},
		{
			color: ui.colors.yellow,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.sklearn.memory },
				{ label: 'flights', x: 1, y: data.flights.sklearn.memory },
				{ label: 'allstate', x: 2, y: data.allstate.sklearn.memory },
			],
			title: 'sklearn',
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
						<ui.TableHeaderCell>{'Memory'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'v. Tangram'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{['higgs', 'flights', 'allstate'].map(dataset =>
						['tangram', 'lightgbm', 'xgboost', 'sklearn'].map(library => (
							<ui.TableRow key={dataset + library}>
								<ui.TableCell>{dataset}</ui.TableCell>
								<ui.TableCell>{library}</ui.TableCell>
								<ui.TableCell>
									{formatGB(data[dataset][library].memory)}
								</ui.TableCell>
								<ui.TableCell>
									{formatGBDiff(
										data[dataset][library].memory /
											data[dataset]['tangram'].memory,
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
	let barChartTimeData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.tangram.duration },
				{ label: 'flights', x: 1, y: data.flights.tangram.duration },
				{ label: 'allstate', x: 2, y: data.allstate.tangram.duration },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.lightgbm.duration },
				{ label: 'flights', x: 1, y: data.flights.lightgbm.duration },
				{ label: 'allstate', x: 2, y: data.allstate.lightgbm.duration },
			],
			title: 'lightgbm',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.xgboost.duration },
				{ label: 'flights', x: 1, y: data.flights.xgboost.duration },
				{ label: 'allstate', x: 2, y: data.allstate.xgboost.duration },
			],
			title: 'xgboost',
		},
		{
			color: ui.colors.yellow,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.sklearn.duration },
				{ label: 'flights', x: 1, y: data.flights.sklearn.duration },
				{ label: 'allstate', x: 2, y: data.allstate.sklearn.duration },
			],
			title: 'sklearn',
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
						<ui.TableHeaderCell>{'Training Time'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'v. Tangram'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{['higgs', 'flights', 'allstate'].map(dataset =>
						['tangram', 'lightgbm', 'xgboost', 'sklearn'].map(library => (
							<ui.TableRow key={dataset + library}>
								<ui.TableCell>{dataset}</ui.TableCell>
								<ui.TableCell>{library}</ui.TableCell>
								<ui.TableCell>
									{formatTime(data[dataset][library].duration)}
								</ui.TableCell>
								<ui.TableCell>
									{formatTimeDiff(
										data[dataset][library].duration /
											data[dataset]['tangram'].duration,
									)}
								</ui.TableCell>
							</ui.TableRow>
						)),
					)}
				</ui.TableBody>
			</ui.Table>
			<BarChart
				data={barChartTimeData}
				groupGap={10}
				id="time_benchmark"
				title="Training Time Benchmark (seconds)"
				xAxisTitle="Library"
				yAxisTitle="Training Time (seconds)"
			/>
		</div>
	)
}

function TreeAUCBenchmark() {
	let barChartAUCData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.tangram.auc_roc },
				{ label: 'flights', x: 1, y: data.flights.tangram.auc_roc },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.lightgbm.auc_roc },
				{ label: 'flights', x: 1, y: data.flights.lightgbm.auc_roc },
			],
			title: 'lightgbm',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.xgboost.auc_roc },
				{ label: 'flights', x: 1, y: data.flights.xgboost.auc_roc },
			],
			title: 'xgboost',
		},
		{
			color: ui.colors.yellow,
			data: [
				{ label: 'higgs', x: 0, y: data.higgs.sklearn.auc_roc },
				{ label: 'flights', x: 1, y: data.flights.sklearn.auc_roc },
			],
			title: 'sklearn',
		},
	]
	let formatAUC = (value: number) => `${ui.formatNumber(value, 4)}`
	let formatAUCDiff = (value: number) => `${ui.formatNumber(value, 4)}x`
	return (
		<div className="benchmarks_table_chart_grid">
			<ui.Table>
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Dataset'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Library'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'AUC'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'v. Tangram'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{['higgs', 'flights'].map(dataset =>
						['tangram', 'lightgbm', 'xgboost', 'sklearn'].map(library => (
							<ui.TableRow key={dataset + library}>
								<ui.TableCell>{dataset}</ui.TableCell>
								<ui.TableCell>{library}</ui.TableCell>
								<ui.TableCell>
									{formatAUC(data[dataset][library].auc_roc)}
								</ui.TableCell>
								<ui.TableCell>
									{formatAUCDiff(
										data[dataset][library].auc_roc /
											data[dataset]['tangram'].auc_roc,
									)}
								</ui.TableCell>
							</ui.TableRow>
						)),
					)}
				</ui.TableBody>
			</ui.Table>
			<BarChart
				data={barChartAUCData}
				groupGap={10}
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
