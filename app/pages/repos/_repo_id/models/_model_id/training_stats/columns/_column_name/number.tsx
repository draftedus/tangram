import { BarChart, BoxChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { MetricsRow } from 'common/metrics_row'
import { h } from 'preact'

export type Props = {
	histogram: Array<[number, number]> | null
	invalidCount: number
	max: number
	mean: number
	min: number
	name: string
	p25: number
	p50: number
	p75: number
	std: number
	uniqueCount: number
}

export function NumberColumnDetail(props: Props) {
	let quantilesData = [
		{
			color: ui.colors.blue,
			data: [
				{
					label: props.name,
					x: 0,
					y: {
						max: props.max,
						min: props.min,
						p25: props.p25,
						p50: props.p50,
						p75: props.p75,
					},
				},
			],
			title: 'quartiles',
		},
	]
	let histogramData = props.histogram && [
		{
			color: ui.colors.blue,
			data: props.histogram.map(([label, count], i) => ({
				label: ui.formatNumber(label),
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
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart title="Mean" value={ui.formatNumber(props.mean)} />
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							title="Standard Deviation"
							value={ui.formatNumber(props.std)}
						/>
					</ui.Card>
				</MetricsRow>
				<MetricsRow>
					<ui.Card>
						<ui.NumberChart title="Min" value={ui.formatNumber(props.min)} />
					</ui.Card>
					<ui.Card>
						<ui.NumberChart title="p25" value={ui.formatNumber(props.p25)} />
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							title="p50 (median)"
							value={ui.formatNumber(props.p50)}
						/>
					</ui.Card>
					<ui.Card>
						<ui.NumberChart title="p75" value={ui.formatNumber(props.p75)} />
					</ui.Card>
					<ui.Card>
						<ui.NumberChart title="Max" value={ui.formatNumber(props.max)} />
					</ui.Card>
				</MetricsRow>
				{quantilesData && (
					<ui.Card>
						<BoxChart
							data={quantilesData}
							id="number_quantiles"
							title={`Distribution of Values for ${props.name}`}
						/>
					</ui.Card>
				)}
				{histogramData && (
					<ui.Card>
						<BarChart
							data={histogramData}
							id="number_histogram"
							shouldDrawXAxisLabels={true}
							title={`Histogram of Unique Values for ${props.name}`}
							xAxisTitle={props.name}
							yAxisTitle="Count"
						/>
					</ui.Card>
				)}
				{props.histogram && (
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>{'Value'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Count'}</ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.histogram.map(([value, count], i) => (
								<ui.TableRow key={i}>
									<ui.TableCell>{ui.formatNumber(value)}</ui.TableCell>
									<ui.TableCell>{count}</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				)}
			</ui.S2>
		</ui.S1>
	)
}
