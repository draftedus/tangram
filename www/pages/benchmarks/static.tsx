import { BarChart } from '@tangramhq/charts'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { PageLayout } from 'layouts/page_layout'
import { h } from 'preact'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function Home(props: Props) {
	return renderPage(
		<PageLayout background={true} pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Benchmarks'}</ui.H1>
				<ui.P>
					{
						'Under the hood, the Tangram CLI uses Tangram Tree and Tangram Linear to train machine learning models. Tangram tree is the fastest gradient boosted decision library in the world and has the smallest memory footprint. Below are benchmarks comparing auc, training times and memory usage on three very large datasets across three machine learing libraries: xgboost, lightgbm and tangram.'
					}
				</ui.P>
				<Benchmarks />
			</ui.S1>
		</PageLayout>,
	)
}

function Benchmarks() {
	let barChartMemoryData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: 2455276 },
				{ label: 'flights', x: 1, y: 1140732 },
				{ label: 'allstate', x: 2, y: 4831324 },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: 11618204 },
				{ label: 'flights', x: 1, y: 2007908 },
				{ label: 'allstate', x: 2, y: 12372716 },
			],
			title: 'lightgbm',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: 12743936 },
				{ label: 'flights', x: 1, y: null },
				{ label: 'allstate', x: 2, y: null },
			],
			title: 'xgboost',
		},
	]

	let barChartTimeData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: 100.176465602 },
				{ label: 'flights', x: 1, y: 37.496518463 },
				{ label: 'allstate', x: 2, y: 62.931978142 },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: 124.955734558 },
				{ label: 'flights', x: 1, y: 45.382501249 },
				{ label: 'allstate', x: 2, y: 69.314861553 },
			],
			title: 'lightgbm',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: 101.76733798 },
				{ label: 'flights', x: 1, y: null },
				{ label: 'allstate', x: 2, y: null },
			],
			title: 'xgboost',
		},
	]

	let barChartAUCData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: 'higgs', x: 0, y: 0.8320089 },
				{ label: 'flights', x: 1, y: 0.7815357 },
			],
			title: 'tangram',
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: 'higgs', x: 0, y: 0.83145106 },
				{ label: 'flights', x: 1, y: 0.7807312 },
			],
			title: 'lightgbm',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: 'higgs', x: 0, y: 0.81292254 },
				{ label: 'flights', x: 1, y: null },
			],
			title: 'xgboost',
		},
	]
	return (
		<div>
			<BarChart
				data={barChartMemoryData}
				groupGap={30}
				id="memory_benchmark"
				title="Memory Usage Benchmark (kB)"
				xAxisTitle="Library"
				yAxisTitle="Memory Usage (kB)"
			/>
			<BarChart
				data={barChartTimeData}
				groupGap={30}
				id="time_benchmark"
				title="Training Time Benchmark (seconds)"
				xAxisTitle="Library"
				yAxisTitle="Training Time (seconds)"
			/>
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
