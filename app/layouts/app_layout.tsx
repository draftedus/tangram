import './app_layout.css'
import { TopbarLayout } from './topbar_layout'
import { ComponentChildren, h } from 'preact'

type AppLayoutProps = {
	children?: ComponentChildren
	clientJsSrc?: string
	cssSrcs?: string[]
	preloadJsSrcs?: string[]
	// topbarAvatar: TopbarAvatar | null
}

export function AppLayout(props: AppLayoutProps) {
	return (
		<TopbarLayout
			clientJsSrc={props.clientJsSrc}
			cssSrcs={props.cssSrcs}
			preloadJsSrcs={props.preloadJsSrcs}
			// topbarAvatar={props.topbarAvatar}
		>
			<div class="app-layout">{props.children}</div>
		</TopbarLayout>
	)
}
