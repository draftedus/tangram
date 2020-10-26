import { AppLayout, AppLayoutInfo } from './app_layout'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { ComponentChildren, h } from 'preact'

type Props = {
	appLayoutInfo: AppLayoutInfo
	children?: ComponentChildren
	id: string
	name: string
	pageInfo: PageInfo
}

export function OrganizationLayout(props: Props) {
	return (
		<AppLayout info={props.appLayoutInfo} pageInfo={props.pageInfo}>
			<ui.S1>
				<ui.H1>{props.name}</ui.H1>
				<ui.TabBar>
					<ui.TabLink href={`/organizations/${props.id}/`}>
						{'Overview'}
					</ui.TabLink>
					<ui.TabLink href={`/organizations/${props.id}/plan`}>
						{'Plan'}
					</ui.TabLink>
					<ui.TabLink href={`/organizations/${props.id}/billing`}>
						{'Billing'}
					</ui.TabLink>
				</ui.TabBar>
				{props.children}
			</ui.S1>
		</AppLayout>
	)
}
