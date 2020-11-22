import { TextProps } from "./props"
import { BarChart } from "@tangramhq/charts"
import * as ui from "@tangramhq/ui"
import { h } from "preact"

export function TextColumnDetail(props: TextProps) {
	let data = [
		{
			color: ui.colors.blue,
			data: props.tokens.map((token, i) => ({
				label: token.token,
				x: i,
				y: token.count,
			})),
			title: "Token Count",
		},
	]
	return (
		<ui.S1>
			<ui.H1>{props.name}</ui.H1>
			<ui.S2>
				<ui.Card>
					<BarChart
						id="token_histogram"
						series={data}
						shouldDrawXAxisLabels={false}
						title="Most Frequent Tokens"
						xAxisTitle="Token"
						yAxisTitle="Count"
					/>
				</ui.Card>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableHeaderCell>{"Token"}</ui.TableHeaderCell>
						<ui.TableHeaderCell>{"Count"}</ui.TableHeaderCell>
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
