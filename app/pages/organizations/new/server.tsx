import { Props } from "./props"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { renderPage } from "common/render"
import { AppLayout } from "layouts/app_layout"
import { h } from "preact"

export default (pageInfo: PageInfo, props: Props) => {
	return renderPage(
		<AppLayout info={props.appLayoutInfo} pageInfo={pageInfo}>
			<ui.S1>
				<ui.H1>{"Create New Organization"}</ui.H1>
				<ui.Form post={true}>
					<ui.TextField label="Name" name="name" required={true}></ui.TextField>
					<ui.Button>{"Create"}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
