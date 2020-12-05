import { Install } from "../shared/install"
import { LibraryInstall } from "../shared/library_install"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { renderPage } from "common/render"
import { DocsLayout, DocsPage } from "layouts/docs_layout"
import { h } from "preact"

export default (pageInfo: PageInfo) => {
	return renderPage(
		<DocsLayout pageInfo={pageInfo} selectedPage={DocsPage.Install}>
			<ui.S1>
				<ui.H1>{"Install"}</ui.H1>
				<ui.S2>
					<ui.H2>{"Install the Tangram CLI"}</ui.H2>
					<Install />
				</ui.S2>
				<ui.S2>
					<ui.H2>
						{"Install the Tangram library for your programming language"}
					</ui.H2>
					<LibraryInstall />
				</ui.S2>
			</ui.S1>
		</DocsLayout>,
	)
}
