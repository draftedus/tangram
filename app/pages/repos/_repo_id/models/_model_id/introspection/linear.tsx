import { h, ui } from 'deps'

type Props = {
	values: Array<[string, number]>
}

export function LinearFeatureWeights(props: Props) {
	let max = Math.max(...props.values.map(([, value]) => Math.abs(value)))
	let data = [
		{
			color: ui.colors.blue,
			data: props.values.map(([label, value], i) => ({
				label,
				x: i,
				y: value,
			})),
			title: 'Feature Weight',
		},
	]
	return (
		<ui.S2>
			<ui.H2>{'Feature Weights'}</ui.H2>
			<ui.Card>
				<ui.BarChart
					data={data}
					id="feature_weights"
					shouldDrawXAxisLabels={false}
					title="Feature Weights"
					xAxisTitle="Name"
					yAxisTitle="Weight"
				/>
			</ui.Card>
			<ui.Table width="100%">
				<ui.TableHeader>
					<ui.TableHeaderCell>{'Feature'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'Weight'}</ui.TableHeaderCell>
				</ui.TableHeader>
				<ui.TableBody>
					{props.values.map(([feature, weight], i) => (
						<ui.TableRow key={i}>
							<ui.TableCell>{feature}</ui.TableCell>
							<ui.TableCell
								color={
									(weight > 0 ? 'var(--green)' : 'var(--red)') +
									Math.floor((Math.abs(weight) / max) * 255)
										.toString(16)
										.padStart(2, '0')
								}
							>
								{weight}
							</ui.TableCell>
						</ui.TableRow>
					))}
				</ui.TableBody>
			</ui.Table>
		</ui.S2>
	)
}
