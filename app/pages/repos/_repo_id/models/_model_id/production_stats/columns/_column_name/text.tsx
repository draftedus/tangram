import { TextProps } from "./props"
import { BarChart } from "@tangramhq/charts"
import * as ui from "@tangramhq/ui"
import { MetricsRow } from "common/metrics_row"
import { overallChartTitle } from "common/time"
import { productionColor } from "common/tokens"
import { h } from "preact"

export function Text(props: TextProps) {
	let overallChartData = [
		{
			color: productionColor,
			data: props.overallTokenHistogram.map(([label, count], i) => ({
				label,
				x: i,
				y: count,
			})),
			title: "Production",
		},
	]
	let overallDistributionChartTitle = overallChartTitle(
		props.dateWindow,
		`Distribution of Unique Values for ${props.columnName}`,
	)
	return (
		<ui.S2>
			{props.alert && (
				<ui.Alert level={ui.Level.Danger}>{props.alert}</ui.Alert>
			)}
			<ui.Card>
				<BarChart
					data={overallChartData}
					id="text_overall"
					title={overallDistributionChartTitle}
					xAxisTitle={props.columnName}
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
			<ui.H2>{"Unique Tokens"}</ui.H2>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{"Token"}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{"Count"}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{props.overallTokenHistogram.map(([value, count]) => (
						<ui.TableRow key={value}>
							<ui.TableCell>{value}</ui.TableCell>
							<ui.TableCell>{ui.formatNumber(count)}</ui.TableCell>
						</ui.TableRow>
					))}
				</ui.TableBody>
			</ui.Table>
		</ui.S2>
	)
}
