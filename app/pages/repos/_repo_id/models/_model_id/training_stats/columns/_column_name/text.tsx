import { h, ui } from 'deps'

export type Props = {
	name: string
	tokens: Array<[string, number]>
}

export function TextColumnDetail(props: Props) {
	let data = [
		{
			color: ui.colors.blue,
			data: props.tokens.map(([_value, count], i) => ({
				x: i,
				y: count,
			})),
			title: 'Token Count',
		},
	]
	return (
		<ui.S1>
			<ui.H1>{props.name}</ui.H1>
			<ui.S2>
				<ui.Card>
					<ui.BarChart
						data={data}
						title="Most Frequent Tokens"
						xAxisLabelFormatter={i => props.tokens[i][0]}
						xAxisTitle="Token"
						yAxisTitle="Count"
					/>
				</ui.Card>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableHeaderCell>{'Token'}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{'Count'}</ui.TableHeaderCell>
					</ui.TableHeader>
					<ui.TableBody>
						{props.tokens.map(([token, count], i) => (
							<ui.TableRow key={i}>
								<ui.TableCell>{token}</ui.TableCell>
								<ui.TableCell>{count}</ui.TableCell>
							</ui.TableRow>
						))}
					</ui.TableBody>
				</ui.Table>
			</ui.S2>
		</ui.S1>
	)
}
