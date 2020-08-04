import { MetricsRow } from 'common/metrics_row'
import {
	DateWindow,
	DateWindowInterval,
	overallChartTitle,
} from 'common/time_charts'
import { productionColor } from 'common/tokens'
import { h, ui } from 'deps'

export type Props = {
	alert: string | null
	dateWindow: DateWindow
	dateWindowInterval: DateWindowInterval
	name: string
	overall: {
		absentCount: number
		invalidCount: number
		label: string
		rowCount: number
		tokenHistogram: Array<[string, number]>
	}
}

export function Text(props: Props) {
	let overallChartData = [
		{
			color: productionColor,
			data: props.overall.tokenHistogram.map(([_value, count], i) => ({
				x: i,
				y: count,
			})),
			title: 'Production',
		},
	]
	let overallDistributionChartTitle = overallChartTitle(
		props.dateWindow,
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
					title={overallDistributionChartTitle}
					xAxisLabelFormatter={x => props.overall.tokenHistogram[x][0]}
					xAxisTitle={props.name}
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
			<ui.H2>{'Unique Tokens'}</ui.H2>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableRow>
						<ui.TableHeaderCell>{'Token'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Count'}</ui.TableHeaderCell>
					</ui.TableRow>
				</ui.TableHeader>
				<ui.TableBody>
					{props.overall.tokenHistogram.map(([value, count]) => (
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
