import './app_layout.css'
import { PageInfo } from '@tangramhq/pinwheel'
import { Document } from '@tangramhq/www'
import { Topbar, TopbarAvatar } from 'common/topbar'
import { ComponentChildren, h } from 'preact'

type AppLayoutProps = {
	children?: ComponentChildren
	info: AppLayoutInfo
	pageInfo: PageInfo
}

export type AppLayoutInfo = {
	topbarAvatar: TopbarAvatar | null
}

export function AppLayout(props: AppLayoutProps) {
	return (
		<Document pageInfo={props.pageInfo}>
			<div class="app-layout-topbar-grid">
				<Topbar topbarAvatar={props.info.topbarAvatar} />
				<div class="app-layout">{props.children}</div>
			</div>
		</Document>
	)
}
