import { Props } from './props'
import { Tuning } from './tuning'
import { Client } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

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
