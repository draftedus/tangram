import { Tuning, TuningProps } from './tuning'
import { Client, PinwheelInfo, h, renderPage, ui } from 'deps'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'

export type Props = {
	modelLayoutInfo: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	tuning: TuningProps | null
}

export default function TuningPage(props: Props) {
	let inner
	if (props.tuning) {
		inner = <Client component={Tuning} id="tuning" props={props.tuning} />
	} else {
		inner = (
			<ui.S1>
				<ui.P>{'Tuning is not supported for this model.'}</ui.P>
			</ui.S1>
		)
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.Tuning}
		>
			{inner}
		</ModelLayout>,
	)
}
