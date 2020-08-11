import { Install } from '../shared/install'
import { LibraryInstall } from '../shared/library_install'
import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { DocsLayout } from 'layouts/docs_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default (props: Props) =>
	renderPage(
		<DocsLayout pagename="/docs/install" pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Install'}</ui.H1>
				<ui.S2>
					<ui.H2>{'Install the Tangram CLI'}</ui.H2>
					<Install />
				</ui.S2>
				<ui.S2>
					<ui.H2>
						{'Install the Tangram library for your programming language'}
					</ui.H2>
					<LibraryInstall />
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
