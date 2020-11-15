import { NumberProps } from "./props"
import { BoxChart } from "@tangramhq/charts"
import * as ui from "@tangramhq/ui"
import { MetricsRow } from "common/metrics_row"
import { h } from "preact"

export function NumberColumnDetail(props: NumberProps) {
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
			title: "quartiles",
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
			</ui.S2>
		</ui.S1>
	)
}
