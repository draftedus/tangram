import { BarChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export type Props = {
	values: Array<[string, number]>
}

export function TreeFeatureImportances(props: Props) {
	let max = Math.max(...props.values.map(([, value]) => Math.abs(value)))
	let data = [
		{
			color: ui.colors.blue,
			data: props.values
				.filter(([_, value]) => value > 0)
				.map(([label, value], i) => ({
					label,
					x: i,
					y: value,
				})),
			title: 'Feature Importance',
		},
	]
	return (
		<ui.S2>
			<ui.H2>{'Feature Importances'}</ui.H2>
			<ui.Card>
				<BarChart
					data={data}
					id="feature_importances"
					shouldDrawXAxisLabels={false}
					title="Feature Importances"
					xAxisTitle="Feature Name"
					yAxisTitle="Importance Score"
				/>
			</ui.Card>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableHeaderCell>{'Feature'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'Importance'}</ui.TableHeaderCell>
				</ui.TableHeader>
				<ui.TableBody>
					{props.values.map(([feature, weight], i) => (
						<ui.TableRow key={i}>
							<ui.TableCell>{feature}</ui.TableCell>
							<ui.TableCell>{weight / max}</ui.TableCell>
						</ui.TableRow>
					))}
				</ui.TableBody>
			</ui.Table>
		</ui.S2>
	)
}
