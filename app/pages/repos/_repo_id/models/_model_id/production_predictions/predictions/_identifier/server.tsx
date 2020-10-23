import { Props } from './props'
import * as ui from '@tangramhq/ui'
import { PredictionResult } from 'common/predict'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default function ProductionPredictionPage(props: Props) {
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.ProductionPredictions}
		>
			<ui.S1>
				<ui.H1>{'Prediction'}</ui.H1>
				<ui.Table width="100%">
					<ui.TableHeader>
						<ui.TableRow>
							<ui.TableHeaderCell>{'Identifier'}</ui.TableHeaderCell>
							<ui.TableHeaderCell>{'Date'}</ui.TableHeaderCell>
						</ui.TableRow>
					</ui.TableHeader>
					<ui.TableBody>
						<ui.TableRow>
							<ui.TableCell>{props.identifier}</ui.TableCell>
							<ui.TableCell>{props.date}</ui.TableCell>
						</ui.TableRow>
					</ui.TableBody>
				</ui.Table>
				<PredictionResult
					inputTable={props.inputTable}
					prediction={props.prediction}
				/>
			</ui.S1>
		</ModelLayout>,
	)
}
