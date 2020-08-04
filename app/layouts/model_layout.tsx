import { TopbarLayout } from './topbar_layout'
import { Children, css, h, r, ui, useCss } from 'deps'

export type ModelLayoutProps = {
	children?: Children
	id: string
	modelId: string
	modelTitle: string
	models: Array<{
		id: string
		isMain: boolean
		title: string
	}>
	ownerName: string
	ownerUrl: string
	pagename: string
	title: string
}

let modelLayoutCss = css({
	[ui.mobile]: {
		[`.model-layout`]: {
			display: 'grid',
			grid: '"model-group-topbar" auto  "model" auto / minmax(0, 1fr)',
			overflow: 'hidden',
		},
	},
	[ui.desktop]: {
		[`.model-layout`]: {
			display: 'grid',
			grid: '"model-group-topbar" auto "model" 1fr / minmax(0, 1fr)',
			height: '100%',
			overflow: 'hidden',
		},
	},
})

let modelGridCss = css({
	[ui.mobile]: {
		[`.model-layout-grid`]: {
			display: 'grid',
			grid: '"side-nav" auto "content" auto / minmax(0, 1fr)',
			gridGap: '2rem',
			height: '100%',
			overflow: 'hidden',
		},
	},
	[ui.desktop]: {
		[`.model-layout-grid`]: {
			display: 'grid',
			grid: '"side-nav content" 1fr / auto minmax(0, 1fr)',
			gridGap: '2rem',
			height: '100%',
			overflow: 'hidden',
		},
	},
})

let modelGroupTopbarCss = css({
	[`.model-layout-model-group-topbar`]: {
		alignItems: 'center',
		backgroundColor: ui.variables.colors.surface,
		display: 'grid',
		grid: 'auto / auto 1fr auto auto',
		gridAutoFlow: 'column',
		gridGap: '1rem',
		padding: '2rem 1rem',
	},
})

let repoTitleCss = css({
	[`.model-layout-repo-title`]: { fontSize: '1.25rem' },
})

let ownerTitleCss = css({
	[`.model-layout-owner-title`]: {
		color: `${ui.variables.colors.text} !important`,
	},
})

let contentCss = css({
	[`.model-layout-content`]: {
		boxSizing: 'border-box',
		margin: '0 auto',
		maxWidth: ui.variables.width.max,
		padding: '2rem 2rem',
		width: '100%',
	},
})

export function ModelLayout(props: ModelLayoutProps) {
	useCss(
		modelLayoutCss,
		modelGridCss,
		modelGroupTopbarCss,
		// modelTopbarCss,
		repoTitleCss,
		ownerTitleCss,
		contentCss,
	)

	let selectedModel = r(props.models.find(model => model.id == props.modelId))
		?.title

	return (
		<TopbarLayout>
			<div class="model-layout">
				<div
					class="model-layout-model-group-topbar"
					style={{ gridArea: 'model-group-topbar' }}
				>
					<div class="model-layout-repo-title">
						<ui.Link
							className="model-layout-owner-title"
							href={props.ownerUrl}
							title="owner"
						>
							{props.ownerName}
						</ui.Link>
						{'/'}
						<ui.Link
							className="model-layout-owner-title"
							href={`/repos/${props.id}/models/${props.modelId}/`}
							title="repo"
						>
							{props.title}
						</ui.Link>
					</div>
					<ui.Details
						options={
							props.models.map(model => ({
								href: `/repos/${props.id}/models/${model.id}/`,
								name: model.title,
							})) ?? []
						}
						summary={selectedModel}
					/>
					<ui.Button
						download={`${props.modelTitle}.tangram`}
						href={`/repos/${props.id}/models/${props.modelId}/download`}
					>
						{'Download Model'}
					</ui.Button>
					<ui.Button href={`/repos/${props.id}/models/new`}>
						{'Upload new model version'}
					</ui.Button>
				</div>
				<div class="model-layout-grid" style={{ gridArea: 'model' }}>
					<div
						class="model-layout-sidenav-wrapper"
						style={{ gridArea: 'side-nav' }}
					>
						<ModelSideNav
							id={props.modelId}
							pagename={props.pagename}
							title={props.modelTitle}
						/>
					</div>
					<div style={{ gridArea: 'content' }}>
						<div class="model-layout-content">{props.children}</div>
					</div>
				</div>
			</div>
		</TopbarLayout>
	)
}

type ModelSideNavProps = {
	id: string
	pagename: string
	title: string
}

function ModelSideNav(props: ModelSideNavProps) {
	return (
		<ui.SideNav>
			<ui.SideNavSection>
				<ui.SideNavItem href={`/repos/${props.id}/models/${props.id}/`}>
					{'Overview'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_stats/`}
					// selected={props.pagename.startsWith(
					// 	`/models/_model_id/training_stats/`,
					// )}
				>
					{'Training Stats'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_metrics/`}
					// selected={props.pagename.startsWith(
					// 	`/models/_model_id/training_metrics/`,
					// )}
				>
					{'Training Metrics'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/introspect`}
				>
					{'Introspect'}
				</ui.SideNavItem>
				<ui.SideNavItem href={`/repos/${props.id}/models/${props.id}/predict`}>
					{'Predict'}
				</ui.SideNavItem>
				<ui.SideNavItem href={`/repos/${props.id}/models/${props.id}/tuning`}>
					{'Tuning'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_stats/`}
					// selected={props.pagename.startsWith(
					// 	`/models/_model_id/production_stats/`,
					// )}
				>
					{'Production Stats'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_metrics/`}
					// selected={props.pagename.startsWith(
					// 	`/models/_model_id/production_metrics/`,
					// )}
				>
					{'Production Metrics'}
				</ui.SideNavItem>
			</ui.SideNavSection>
		</ui.SideNav>
	)
}
