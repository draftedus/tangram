import { AppLayout } from './layouts/app_layout'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export default (props: Props) => {
	return (
		<AppLayout
			clientJsSrc={props.clientJsSrc}
			cssSrcs={props.cssSrcs}
			preloadJsSrcs={props.preloadJsSrcs}
		>
			<ui.S1>
				<ui.H1>{'Not Found'}</ui.H1>
				<ui.P>{'We were unable to find the page you requested.'}</ui.P>
			</ui.S1>
		</AppLayout>
	)
}
