import { TopbarLayout } from './topbar_layout'
import { Children, h } from 'deps'

type AppLayoutProps = { children?: Children }

export function AppLayout(props: AppLayoutProps) {
	return (
		<TopbarLayout>
			<div class="app-layout">{props.children}</div>
		</TopbarLayout>
	)
}
