import { PageLayout } from "./layouts/page_layout"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { h } from "preact"

export default (pageInfo: PageInfo) => {
	return (
		<PageLayout pageInfo={pageInfo}>
			<ui.S1>
				<ui.H1>{"Not Found"}</ui.H1>
				<ui.P>{"We were unable to find the page you requested."}</ui.P>
			</ui.S1>
		</PageLayout>
	)
}
