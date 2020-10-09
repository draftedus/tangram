import * as ui from '@tangramhq/ui'
import * as definitions from 'common/definitions'
import { baselineColor, trainingColor } from 'common/tokens'
import { h } from 'preact'

export type Props = {
	accuracy: number
	baselineAccuracy: number
	classMetrics: Array<{
		precision: number
		recall: number
	}>
	classes: string[]
	id: string
	losses: number[] | null
}

export function MulticlassClassifierTrainingMetricsIndexPage(props: Props) {
	let loss = props.losses?.slice(-1).pop()
	return (
		<ui.S1>
			<ui.H1>{'Training Metrics'}</ui.H1>
			<ui.TabBar>
				<ui.TabLink href="./" selected={true}>
					{'Overview'}
				</ui.TabLink>
				<ui.TabLink href="class_metrics">{'Class Metrics'}</ui.TabLink>
			</ui.TabBar>
			<ui.S2>
				<ui.P>
					{
						'At the end of training, your model was evaluated on a test dataset. All metrics in this section are from that evaluation. You can use these metrics to see how your model might perform on unseen production data.'
					}
				</ui.P>
			</ui.S2>
			<ui.S2>
				<ui.H2>{'Accuracy'}</ui.H2>
				<ui.P>{definitions.accuracy}</ui.P>
				<ui.Card>
					<ui.NumberComparisonChart
						colorA={baselineColor}
						colorB={trainingColor}
						title="Accuracy"
						valueA={props.baselineAccuracy}
						valueATitle="Baseline Accuracy"
						valueB={props.accuracy}
						valueBTitle="Accuracy"
						valueFormatter={value => ui.formatPercent(value, 2)}
					/>
				</ui.Card>
			</ui.S2>
			<ui.S2>
				<ui.H2>{'Precision and Recall'}</ui.H2>
				<ui.P>{definitions.precisionRecall}</ui.P>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableRow>
							<ui.TableHeaderCell>{'Class'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Precision'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Recall'}</ui.TableHeaderCell>
						</ui.TableRow>
					</ui.TableHeader>
					<ui.TableBody>
						{props.classes.map((className, i) => (
							<ui.TableRow key={className}>
								<ui.TableCell>{className}</ui.TableCell>
								<ui.TableCell>
									{ui.formatPercent(props.classMetrics[i].precision, 2)}
								</ui.TableCell>
								<ui.TableCell>
									{ui.formatPercent(props.classMetrics[i].recall, 2)}
								</ui.TableCell>
							</ui.TableRow>
						))}
					</ui.TableBody>
				</ui.Table>
			</ui.S2>
			{loss !== undefined && (
				<ui.S2>
					<ui.H2>{'Loss'}</ui.H2>
					<ui.P>{definitions.crossEntropyLoss}</ui.P>
					<ui.Card>
						<ui.NumberChart
							title="Cross Entropy Loss"
							value={ui.formatNumber(loss)}
						/>
					</ui.Card>
				</ui.S2>
			)}
		</ui.S1>
	)
}
