import { Enum } from './enum'
import { Number } from './number'
import { Props, Type } from './props'
import { Text } from './text'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { DateWindowSelectField } from 'common/date_window_select_field'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	let inner
	switch (props.inner.type) {
		case Type.Number:
			inner = <Number {...props.inner.value} />
			break
		case Type.Enum:
			inner = <Enum {...props.inner.value} />
			break
		case Type.Text:
			inner = <Text {...props.inner.value} />
			break
	}
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pageInfo={pageInfo}
			selectedItem={ModelSideNavItem.ProductionStats}
		>
			<ui.S1>
				<ui.H1>{props.columnName}</ui.H1>
				<ui.Form>
					<DateWindowSelectField dateWindow={props.dateWindow} />
					<noscript>
						<ui.Button>{'Submit'}</ui.Button>
					</noscript>
				</ui.Form>
				{inner}
			</ui.S1>
		</ModelLayout>,
	)
}
