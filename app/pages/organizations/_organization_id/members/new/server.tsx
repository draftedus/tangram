import { Props } from './props'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	return (
		<AppLayout
			clientJsSrc={pageInfo.clientJsSrc}
			cssSrcs={pageInfo.cssSrcs}
			preloadJsSrcs={pageInfo.preloadJsSrcs}
		>
			<ui.S1>
				<ui.H1>{'Invite Team Member'}</ui.H1>
				<ui.Form post={true}>
					<ui.TextField label="Email" name="email"></ui.TextField>
					<ui.CheckboxField label="Admin" name="isAdmin" />
					<ui.Button>{'Invite'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
