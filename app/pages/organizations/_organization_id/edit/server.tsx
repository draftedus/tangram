import { Props } from './props'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export default function OrganizationEditPage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Edit Organization'}</ui.H1>
				<ui.Form post={true}>
					<ui.TextField label="Organization Name" name="name" />
					<ui.Button type="submit">{'Submit'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
