import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { DocsLayout, DocsPage } from 'layouts/docs_layout'
import { h } from 'preact'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default (props: Props) =>
	renderPage(
		<DocsLayout pinwheelInfo={props.pinwheelInfo} selectedPage={DocsPage.Home}>
			<ui.S1>
				<ui.H1>{'Docs'}</ui.H1>
				<ui.P>
					{'Watch the video or head over to the '}
					<ui.Link href="/docs/getting-started/train">
						{'getting started'}
					</ui.Link>
					{' section to train your first model.'}
				</ui.P>
			</ui.S1>
		</DocsLayout>,
	)
