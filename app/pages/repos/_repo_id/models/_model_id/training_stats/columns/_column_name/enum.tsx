import { MetricsRow } from 'common/metrics_row'
import { h, ui } from 'deps'

export type Props = {
	histogram: Array<[string, number]>
	invalidCount: number
	name: string
	uniqueCount: number
}

export function EnumColumnDetail(props: Props) {
	let histogramData = [
		{
			color: ui.colors.blue,
			data: props.histogram.map(([_value, count], i) => ({
				x: i,
				y: count,
			})),
			title: 'Unique Values',
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
					<ui.BarChart
						data={histogramData}
						id="enum_histogram"
						title={`Histogram of Unique Values for ${props.name}`}
						xAxisLabelFormatter={x => props.histogram[x][0]}
						xAxisTitle={props.name}
						yAxisTitle="Count"
					/>
				</ui.Card>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableRow>
							<ui.TableHeaderCell>{'Value'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Count'}</ui.TableHeaderCell>
						</ui.TableRow>
					</ui.TableHeader>
					<ui.TableBody>
						{props.histogram.map(([_value, count], i) => (
							<ui.TableRow key={i}>
								<ui.TableCell>{props.histogram[i][0]}</ui.TableCell>
								<ui.TableCell>{count}</ui.TableCell>
							</ui.TableRow>
						))}
					</ui.TableBody>
				</ui.Table>
			</ui.S2>
		</ui.S1>
	)
}
