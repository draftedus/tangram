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
				<ui.H1>{"Upload Model"}</ui.H1>
				<ui.Form encType="multipart/form-data" post={true}>
					{props.error && (
						<ui.Alert level={ui.Level.Danger}>{props.error}</ui.Alert>
					)}
					<ui.FileField label="File" name="file" required={true} />
					<ui.Button type="submit">{"Upload"}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
