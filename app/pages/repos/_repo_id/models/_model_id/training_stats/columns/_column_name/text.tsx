import { BarChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export type Props = {
	name: string
	tokens: TokenStats[]
}

type TokenStats = {
	count: number
	examples_count: number
	token: string
}

export function TextColumnDetail(props: Props) {
	let data = [
		{
			color: ui.colors.blue,
			data: props.tokens.map((token, i) => ({
				label: token.token,
				x: i,
				y: token.count,
			})),
			title: 'Token Count',
		},
	]
	return (
		<ui.S1>
			<ui.H1>{props.name}</ui.H1>
			<ui.S2>
				<ui.Card>
					<BarChart
						data={data}
						id="text_histogram"
						shouldDrawXAxisLabels={false}
						title="Most Frequent Tokens"
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
						{props.tokens.map((token, i) => (
							<ui.TableRow key={i}>
								<ui.TableCell>{token.token}</ui.TableCell>
								<ui.TableCell>{token.count}</ui.TableCell>
							</ui.TableRow>
						))}
					</ui.TableBody>
				</ui.Table>
			</ui.S2>
		</ui.S1>
	)
}
