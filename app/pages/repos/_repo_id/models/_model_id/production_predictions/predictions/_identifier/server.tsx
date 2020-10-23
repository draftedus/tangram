import { Props } from './props'
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
			<PredictionResult
				inputTable={props.inputTable}
				prediction={props.prediction}
			/>
		</ModelLayout>,
	)
}
