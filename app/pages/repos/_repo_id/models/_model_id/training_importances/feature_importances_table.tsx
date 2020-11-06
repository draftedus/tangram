import { FeatureImportance } from './props'
import { BarChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { Fragment, h } from 'preact'

export type Props = {
	featureImportances: FeatureImportance[]
}

export function FeatureImportancesTable(props: Props) {
	let data = [
		{
			color: ui.colors.blue,
			data: props.featureImportances.map(
				({ featureImportanceValue, featureName }, i) => ({
					label: featureName,
					x: i,
					y: featureImportanceValue,
				}),
			),
			title: 'Feature Importance',
		},
	]
	return (
		<>
			<ui.Card>
				<BarChart
					data={data}
					id="feature_importances"
					shouldDrawXAxisLabels={false}
					title="Feature Importances"
					xAxisTitle="Feature Name"
					yAxisTitle="Feature Importance Value"
				/>
			</ui.Card>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableHeaderCell>{'Feature Name'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'Feature Importance Value'}</ui.TableHeaderCell>
				</ui.TableHeader>
				<ui.TableBody>
					{props.featureImportances.map(
						({ featureImportanceValue, featureName }, i) => (
							<ui.TableRow key={i}>
								<ui.TableCell>{featureName}</ui.TableCell>
								<ui.TableCell>{featureImportanceValue}</ui.TableCell>
							</ui.TableRow>
						),
					)}
				</ui.TableBody>
			</ui.Table>
		</>
	)
}
