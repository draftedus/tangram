import { BarChart } from "@tangramhq/charts"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { renderPage } from "common/render"
import { PageLayout } from "layouts/page_layout"
import { h } from "preact"

export default (pageInfo: PageInfo) => {
	return renderPage(
		<PageLayout background={true} pageInfo={pageInfo}>
			<ui.S1>
				<ui.H1>{"Benchmarks"}</ui.H1>
				<ui.P>
					{
						"Under the hood, the Tangram CLI uses Tangram Tree and Tangram Linear to train machine learning models. Tangram tree is the fastest gradient boosted decision tree library and has by far lowest memory usage."
					}
				</ui.P>
				<ui.S2>
					<ui.H2>{"Area Under the Receiver Operating Characteristic"}</ui.H2>
					<TreeAUCBenchmark />
				</ui.S2>
				<ui.S2>
					<ui.H2>{"Time"}</ui.H2>
					<TimeBenchmark />
				</ui.S2>
				<ui.S2>
					<ui.H2>{"Memory Usage"}</ui.H2>
					<MemoryBenchmark />
				</ui.S2>
			</ui.S1>
		</PageLayout>,
	)
}

function TreeAUCBenchmark() {
	let barChartAUCData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.tangram.auc_roc },
				{ label: "flights", x: 1, y: data.flights.tangram.auc_roc },
			],
			title: "tangram",
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.lightgbm.auc_roc },
				{ label: "flights", x: 1, y: data.flights.lightgbm.auc_roc },
			],
			title: "lightgbm",
		},
		{
			color: ui.colors.green,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.xgboost.auc_roc },
				{ label: "flights", x: 1, y: data.flights.xgboost.auc_roc },
			],
			title: "xgboost",
		},
		{
			color: ui.colors.yellow,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.sklearn.auc_roc },
				{ label: "flights", x: 1, y: data.flights.sklearn.auc_roc },
			],
			title: "sklearn",
		},
		{
			color: ui.colors.orange,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.h2o.auc_roc },
				{ label: "flights", x: 1, y: data.flights.h2o.auc_roc },
			],
			title: "h2o",
		},
		{
			color: ui.colors.red,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.catboost.auc_roc },
				{ label: "flights", x: 1, y: data.flights.catboost.auc_roc },
			],
			title: "catboost",
		},
	]
	return (
		<div className="benchmarks_tables_grid_wrapper">
			<div className="benchmarks_tables_grid">
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Flights"}</div>
					<AUCTable data={data.flights} />
				</div>
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Higgs"}</div>
					<AUCTable data={data.higgs} />
				</div>
			</div>
			<BarChart
				data={barChartAUCData}
				groupGap={10}
				id="auc_benchmark"
				title="AUC (higher is better)"
				xAxisTitle="Library"
				yAxisTitle="AUC"
				yMax={1}
				yMin={0}
			/>
		</div>
	)
}

type AUCTableProps<D extends DatasetsForTask[Task.BinaryClassification]> = {
	data: BenchmarkDataForDataset<D>
}

function AUCTable<D extends DatasetsForTask[Task.BinaryClassification]>(
	props: AUCTableProps<D>,
) {
	let formatAUC = (value: number) => `${ui.formatNumber(value, 4)}`
	let formatAUCDiff = (value: number) => `${ui.formatNumber(value, 4)}x`
	return (
		<ui.Table width="100%">
			<ui.TableHeader>
				<ui.TableRow>
					<ui.TableHeaderCell>{"Library"}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{"AUC"}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{"v. Tangram"}</ui.TableHeaderCell>
				</ui.TableRow>
			</ui.TableHeader>
			<ui.TableBody>
				{Object.values(Library).map(library => (
					<ui.TableRow
						color={library == "tangram" ? ui.colors.blue : undefined}
						key={library}
					>
						<ui.TableCell>{library}</ui.TableCell>
						<ui.TableCell>
							{formatAUC(props.data[library].auc_roc)}
						</ui.TableCell>
						<ui.TableCell>
							{formatAUCDiff(
								props.data[library].auc_roc / props.data["tangram"].auc_roc,
							)}
						</ui.TableCell>
					</ui.TableRow>
				))}
			</ui.TableBody>
		</ui.Table>
	)
}

function TimeBenchmark() {
	let barChartTimeData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.tangram.duration },
				{ label: "flights", x: 1, y: data.flights.tangram.duration },
				{ label: "allstate", x: 2, y: data.allstate.tangram.duration },
			],
			title: "tangram",
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.lightgbm.duration },
				{ label: "flights", x: 1, y: data.flights.lightgbm.duration },
				{ label: "allstate", x: 2, y: data.allstate.lightgbm.duration },
			],
			title: "lightgbm",
		},
		{
			color: ui.colors.green,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.xgboost.duration },
				{ label: "flights", x: 1, y: data.flights.xgboost.duration },
				{ label: "allstate", x: 2, y: data.allstate.xgboost.duration },
			],
			title: "xgboost",
		},
		{
			color: ui.colors.yellow,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.sklearn.duration },
				{ label: "flights", x: 1, y: data.flights.sklearn.duration },
				{ label: "allstate", x: 2, y: data.allstate.sklearn.duration },
			],
			title: "sklearn",
		},
		{
			color: ui.colors.orange,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.h2o.duration },
				{ label: "flights", x: 1, y: data.flights.h2o.duration },
				{ label: "allstate", x: 2, y: data.allstate.h2o.duration },
			],
			title: "h2o",
		},
		{
			color: ui.colors.red,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.catboost.duration },
				{ label: "flights", x: 1, y: data.flights.catboost.duration },
				{ label: "allstate", x: 2, y: data.allstate.catboost.duration },
			],
			title: "catboost",
		},
	]
	return (
		<div className="benchmarks_tables_grid_wrapper">
			<div className="benchmarks_tables_grid">
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Allstate"}</div>
					<TimeTable data={data["allstate"]} />
				</div>
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Flights"}</div>
					<TimeTable data={data["flights"]} />
				</div>
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Higgs"}</div>
					<TimeTable data={data["higgs"]} />
				</div>
			</div>
			<BarChart
				data={barChartTimeData}
				groupGap={10}
				id="time_benchmark"
				title="Training Time (lower is better)"
				xAxisTitle="Library"
				yAxisTitle="Training Time (seconds)"
			/>
		</div>
	)
}

type TimeTableProps = {
	data: any
}

function TimeTable(props: TimeTableProps) {
	let formatTime = (value: number) => `${ui.formatNumber(value, 4)} sec`
	let formatTimeDiff = (value: number) => `${ui.formatNumber(value, 4)}x`
	return (
		<ui.Table width="100%">
			<ui.TableHeader>
				<ui.TableRow>
					<ui.TableHeaderCell>{"Library"}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{"Training"}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{"v. Tangram"}</ui.TableHeaderCell>
				</ui.TableRow>
			</ui.TableHeader>
			<ui.TableBody>
				{Object.values(Library).map(library => (
					<ui.TableRow
						color={library == "tangram" ? ui.colors.blue : undefined}
						key={library}
					>
						<ui.TableCell>{library}</ui.TableCell>
						<ui.TableCell>
							{formatTime(props.data[library].duration)}
						</ui.TableCell>
						<ui.TableCell>
							{formatTimeDiff(
								props.data[library].duration / props.data["tangram"].duration,
							)}
						</ui.TableCell>
					</ui.TableRow>
				))}
			</ui.TableBody>
		</ui.Table>
	)
}

function MemoryBenchmark() {
	let barChartMemoryData = [
		{
			color: ui.colors.blue,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.tangram.memory },
				{ label: "flights", x: 1, y: data.flights.tangram.memory },
				{ label: "allstate", x: 2, y: data.allstate.tangram.memory },
			],
			title: "tangram",
		},
		{
			color: ui.colors.purple,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.lightgbm.memory },
				{ label: "flights", x: 1, y: data.flights.lightgbm.memory },
				{ label: "allstate", x: 2, y: data.allstate.lightgbm.memory },
			],
			title: "lightgbm",
		},
		{
			color: ui.colors.green,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.xgboost.memory },
				{ label: "flights", x: 1, y: data.flights.xgboost.memory },
				{ label: "allstate", x: 2, y: data.allstate.xgboost.memory },
			],
			title: "xgboost",
		},
		{
			color: ui.colors.yellow,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.sklearn.memory },
				{ label: "flights", x: 1, y: data.flights.sklearn.memory },
				{ label: "allstate", x: 2, y: data.allstate.sklearn.memory },
			],
			title: "sklearn",
		},
		{
			color: ui.colors.orange,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.h2o.memory },
				{ label: "flights", x: 1, y: data.flights.h2o.memory },
				{ label: "allstate", x: 2, y: data.allstate.h2o.memory },
			],
			title: "h2o",
		},
		{
			color: ui.colors.red,
			data: [
				{ label: "higgs", x: 0, y: data.higgs.catboost.memory },
				{ label: "flights", x: 1, y: data.flights.catboost.memory },
				{ label: "allstate", x: 2, y: data.allstate.catboost.memory },
			],
			title: "catboost",
		},
	]
	return (
		<div className="benchmarks_tables_grid_wrapper">
			<div className="benchmarks_tables_grid">
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Allstate"}</div>
					<MemoryTable data={data["allstate"]} />
				</div>
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Flights"}</div>
					<MemoryTable data={data["flights"]} />
				</div>
				<div className="benchmarks_table_grid">
					<div className="benchmarks_table_title">{"Higgs"}</div>
					<MemoryTable data={data["higgs"]} />
				</div>
			</div>
			<BarChart
				data={barChartMemoryData}
				groupGap={10}
				id="memory_benchmark"
				title="Memory Usage (lower is better)"
				xAxisTitle="Library"
				yAxisTitle="Memory Usage (GB)"
				yMin={0}
			/>
		</div>
	)
}

type MemoryTableProps<D extends Dataset> = {
	data: BenchmarkDataForDataset<D>
}

function MemoryTable<D extends Dataset>(props: MemoryTableProps<D>) {
	let formatMemory = (value: number) => `${ui.formatNumber(value, 4)} GB`
	let formatMemoryDiff = (value: number) => `${ui.formatNumber(value, 4)}x`
	return (
		<ui.Table>
			<ui.TableHeader>
				<ui.TableRow>
					<ui.TableHeaderCell>{"Library"}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{"Memory"}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{"v. Tangram"}</ui.TableHeaderCell>
				</ui.TableRow>
			</ui.TableHeader>
			<ui.TableBody>
				{Object.values(Library).map(library => (
					<ui.TableRow
						color={library == "tangram" ? ui.colors.blue : undefined}
						key={library}
					>
						<ui.TableCell>{library}</ui.TableCell>
						<ui.TableCell>
							{formatMemory(props.data[library].memory)}
						</ui.TableCell>
						<ui.TableCell>
							{formatMemoryDiff(
								props.data[library].memory / props.data["tangram"].memory,
							)}
						</ui.TableCell>
					</ui.TableRow>
				))}
			</ui.TableBody>
		</ui.Table>
	)
}

enum Library {
	Tangram = "tangram",
	LightGBM = "lightgbm",
	XGBoost = "xgboost",
	SKLearn = "sklearn",
	H2O = "h2o",
	CatBoost = "catboost",
}

enum Dataset {
	Allstate = "allstate",
	Flights = "flights",
	Higgs = "higgs",
}

enum Task {
	Regression = "regression",
	BinaryClassification = "binary_classification",
	MulticlassClassification = "multiclass_classification",
}

type BenchmarkEntryCommon = {
	duration: number
	memory: number
}

type BenchmarkEntryRegression = BenchmarkEntryCommon & {
	mse: number
}

type BenchmarkEntryBinaryClassification = BenchmarkEntryCommon & {
	auc_roc: number
}

type BenchmarkEntryMulticlassClassification = BenchmarkEntryCommon & {
	accuracy: number
}

type DatasetsForTask = {
	[Task.Regression]: Dataset.Allstate
	[Task.BinaryClassification]: Dataset.Flights | Dataset.Higgs
	[Task.MulticlassClassification]: never
}

type TaskForDataset = {
	[Dataset.Allstate]: Task.Regression
	[Dataset.Flights]: Task.BinaryClassification
	[Dataset.Higgs]: Task.BinaryClassification
}

type BenchmarkEntryTypeForTask = {
	[Task.Regression]: BenchmarkEntryRegression
	[Task.BinaryClassification]: BenchmarkEntryBinaryClassification
	[Task.MulticlassClassification]: BenchmarkEntryMulticlassClassification
}

type BenchmarkDataForDataset<D extends Dataset> = {
	[L in Library]: BenchmarkEntryTypeForTask[TaskForDataset[D]]
}

type BenchmarkData = {
	[D in Dataset]: BenchmarkDataForDataset<D>
}

let data: BenchmarkData = {
	allstate: {
		catboost: { duration: 1020.302861637, memory: 18.918908, mse: 1579.626 },
		h2o: {
			duration: 315.6087601184845,
			memory: 20.654428,
			mse: 1579.611798325048,
		},
		lightgbm: { duration: 73.958655182, memory: 12.281084, mse: 1587.0221 },
		sklearn: { duration: 72.647664062, memory: 10.928152, mse: 1583.6423 },
		tangram: { duration: 60.637947846, memory: 4.832316, mse: 1587.8885 },
		xgboost: { duration: 77.592920662, memory: 12.41334, mse: 1581.0436 },
	},
	flights: {
		catboost: { auc_roc: 0.7357335, duration: 490.062091923, memory: 9.852156 },
		h2o: {
			auc_roc: 0.7460383618332509,
			duration: 153.73776364326477,
			memory: 3.676572,
		},
		lightgbm: { auc_roc: 0.7807312, duration: 42.959502341, memory: 1.984692 },
		sklearn: { auc_roc: 0.75876635, duration: 58.51804273, memory: 2.542824 },
		tangram: { auc_roc: 0.7815357, duration: 37.162440456, memory: 1.140836 },
		xgboost: { auc_roc: 0.75779957, duration: 47.069649134, memory: 2.420956 },
	},
	higgs: {
		catboost: {
			auc_roc: 0.81350523,
			duration: 392.363988334,
			memory: 13.218528,
		},
		h2o: {
			auc_roc: 0.8076566606451562,
			duration: 540.6656179428101,
			memory: 21.101324,
		},
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
