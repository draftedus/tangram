import { EnumColumnDetail } from './enum'
import { NumberColumnDetail } from './number'
import { Props, Type } from './props'
import { TextColumnDetail } from './text'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default function TrainingStatsColumnPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Number:
			inner = <NumberColumnDetail {...props.inner.value} />
			break
		case Type.Enum:
			inner = <EnumColumnDetail {...props.inner.value} />
			break
		case Type.Text:
			inner = <TextColumnDetail {...props.inner.value} />
			break
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.TrainingStats}
		>
			{inner}
		</ModelLayout>,
	)
}
