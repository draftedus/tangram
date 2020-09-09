import { AppLayout } from './app_layout'
import { ComponentChildren, PinwheelInfo, h, ui } from 'deps'

type Props = {
	children?: ComponentChildren
	id: string
	name: string
	pinwheelInfo: PinwheelInfo
}

export function OrganizationLayout(props: Props) {
	return (
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
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
