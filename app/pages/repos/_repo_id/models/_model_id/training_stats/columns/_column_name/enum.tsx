import { EnumProps } from "./props"
import { BarChart } from "@tangramhq/charts"
import * as ui from "@tangramhq/ui"
import { MetricsRow } from "common/metrics_row"
import { h } from "preact"

export function EnumColumnDetail(props: EnumProps) {
	let histogramChartSeries = [
		{
			color: ui.colors.blue,
			data: props.histogram.map(([label, count], i) => ({
				label,
				x: i,
				y: count,
			})),
			title: "Unique Values",
		},
	]
	return (
		<ui.S1>
			<ui.H1>{props.name}</ui.H1>
			<ui.S2>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart
							title="Unique Count"
							value={props.uniqueCount.toString()}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							title="Invalid Count"
							value={props.invalidCount.toString()}
						/>
					</ui.Card>
				</MetricsRow>
				<ui.Card>
					<BarChart
						id="enum_histogram"
						series={histogramChartSeries}
						title={`Histogram of Unique Values for ${props.name}`}
						xAxisTitle={props.name}
						yAxisTitle="Count"
					/>
				</ui.Card>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableRow>
							<ui.TableHeaderCell>{"Value"}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{"Count"}</ui.TableHeaderCell>
						</ui.TableRow>
					</ui.TableHeader>
					<ui.TableBody>
						{props.histogram.map(([value, count], i) => (
							<ui.TableRow key={i}>
								<ui.TableCell>{value}</ui.TableCell>
								<ui.TableCell>{count}</ui.TableCell>
							</ui.TableRow>
						))}
					</ui.TableBody>
				</ui.Table>
			</ui.S2>
		</ui.S1>
	)
}
