import { AppLayout } from './app_layout'
import * as ui from '@tangramhq/ui'
import { ComponentChildren, h } from 'preact'

type Props = {
	children?: ComponentChildren
	clientJsSrc?: string
	cssSrcs?: string[]
	id: string
	name: string
	preloadJsSrcs?: string[]
}

export function OrganizationLayout(props: Props) {
	return (
		<AppLayout
			clientJsSrc={props.clientJsSrc}
			cssSrcs={props.cssSrcs}
			preloadJsSrcs={props.preloadJsSrcs}
		>
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
