import { LinearFeatureWeights } from './linear'
import { h, ui } from 'deps'

type Props = {
	biases: number[]
	classes: string[]
	selectedClass: string
	weights: Array<Array<[string, number]>>
}

export function LinearMulticlassClassifierModelPage(props: Props) {
	let selectedClassIndex = props.classes.indexOf(props.selectedClass)
	let selectedWeights = props.weights[selectedClassIndex]
	let bias = ui.formatNumber(props.biases[selectedClassIndex])
	return (
		<ui.S1>
			<ui.H1>Model</ui.H1>
			<ui.SelectField
				label="Select Class"
				options={props.classes}
				value={props.selectedClass}
			/>
			<ui.P>
				The model is a <b>Linear Multiclass Classifier</b>.
			</ui.P>
			<ui.P>
				{'Weights are shown for predicting the class '}
				<b>{props.selectedClass}</b>.
			</ui.P>
			<ui.S2>
				<ui.H2>Bias</ui.H2>
				<ui.Table>
					<ui.TableHeader>
						<ui.TableHeaderCell>Bias</ui.TableHeaderCell>
						<ui.TableBody>
							<ui.TableCell>{bias}</ui.TableCell>
						</ui.TableBody>
					</ui.TableHeader>
				</ui.Table>
			</ui.S2>
			<LinearFeatureWeights values={selectedWeights} />
		</ui.S1>
	)
}
