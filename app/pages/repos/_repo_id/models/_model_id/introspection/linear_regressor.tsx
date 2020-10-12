import { LinearFeatureWeights } from './linear_feature_weights'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

type Props = {
	bias: number
	targetColumnName: string
	weights: Array<[string, number]>
}

export function LinearRegressorModelPage(props: Props) {
	let bias = ui.formatNumber(props.bias)
	return (
		<ui.S1>
			<ui.H1>{'Model'}</ui.H1>
			<ui.P>
				{'The model is a '}
				<b>{'Linear Regressor'}</b>
				{'.'}
			</ui.P>
			<ui.P>
				{'Weights are shown for predicting '}
				<b>{props.targetColumnName}</b>
				{'.'}
			</ui.P>
			<ui.S2>
				<ui.H2>{'Bias'}</ui.H2>
				<ui.Table>
					<ui.TableHeader>
						<ui.TableHeaderCell>{'Bias'}</ui.TableHeaderCell>
						<ui.TableBody>
							<ui.TableCell>{bias}</ui.TableCell>
						</ui.TableBody>
					</ui.TableHeader>
				</ui.Table>
			</ui.S2>
			<LinearFeatureWeights values={props.weights} />
		</ui.S1>
	)
}
