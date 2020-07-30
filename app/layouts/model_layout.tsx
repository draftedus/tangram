import { TopbarLayout } from './topbar_layout'
import { Children, css, cssClass, h, r, ui, useCss } from 'deps'

export type ModelLayoutProps = {
	children?: Children
	modelId: string
	modelTitle: string
	pagename: string
	repoId: string
	repoModels: Array<{
		id: string
		title: string
	}>
	repoOwnerName: string
	repoOwnerUrl: string
	repoTitle: string
}

let modelLayoutClass = cssClass()
let modelLayoutCss = css({
	[ui.mobile]: {
		[`.${modelLayoutClass}`]: {
			display: 'grid',
			grid: '"model-group-topbar" auto  "model" auto / minmax(0, 1fr)',
			overflow: 'hidden',
		},
	},
	[ui.desktop]: {
		[`.${modelLayoutClass}`]: {
			display: 'grid',
			grid: '"model-group-topbar" auto "model" 1fr / minmax(0, 1fr)',
			overflow: 'hidden',
		},
	},
})

let modelGridClass = cssClass()
let modelGridCss = css({
	[ui.mobile]: {
		[`.${modelGridClass}`]: {
			display: 'grid',
			grid:
				'"model-topbar" auto "side-nav" auto "content" auto / minmax(0, 1fr)',
			gridGap: '2rem',
			height: '100%',
			overflow: 'hidden',
			padding: '2rem 2rem',
		},
	},
	[ui.desktop]: {
		[`.${modelGridClass}`]: {
			display: 'grid',
			grid:
				'"model-topbar model-topbar" auto "side-nav content" 1fr / auto minmax(0, 1fr)',
			gridGap: '2rem',
			height: '100%',
			overflow: 'hidden',
			padding: '2rem 2rem',
		},
	},
})

let modelGroupTopbarClass = cssClass()
let modelGroupTopbarCss = css({
	[`.${modelGroupTopbarClass}`]: {
		alignItems: 'center',
		backgroundColor: ui.variables.colors.surface,
		display: 'grid',
		gridAutoFlow: 'column',
		gridGap: '1rem',
		justifyContent: 'space-between',
		padding: '2rem 1rem',
	},
})

let modelTopbarClass = cssClass()
let modelTopbarCss = css({
	[`.${modelTopbarClass}`]: {
		alignItems: 'end',
		display: 'grid',
		grid: 'auto / 1fr auto auto',
		gridGap: '2rem',
	},
})

let repoTitleClass = cssClass()
let repoTitleCss = css({
	[`.${repoTitleClass}`]: { fontSize: '1.25rem' },
})

let ownerTitleClass = cssClass()
let ownerTitleCss = css({
	[`.${ownerTitleClass}`]: {
		color: `${ui.variables.colors.text} !important`,
	},
})

let innerClass = cssClass()
let innerCss = css({
	[`.${innerClass}`]: {
		boxSizing: 'border-box',
		margin: '0 auto',
		maxWidth: ui.variables.width.max,
		width: '100%',
	},
})

export function ModelLayout(props: ModelLayoutProps) {
	useCss(
		modelLayoutCss,
		modelGridCss,
		modelGroupTopbarCss,
		modelTopbarCss,
		repoTitleCss,
		ownerTitleCss,
		innerCss,
	)

	let selectedModel = r(
		props.repoModels.find(model => model.id == props.modelId),
	)?.title

	return (
		<TopbarLayout>
			<div class={modelLayoutClass}>
				<div
					class={modelGroupTopbarClass}
					style={{ gridArea: 'model-group-topbar' }}
				>
					<div class={repoTitleClass}>
						<ui.Link className={ownerTitleClass} href={props.repoOwnerUrl}>
							{props.repoOwnerName}
						</ui.Link>
						/
						<ui.Link
							className={ownerTitleClass}
							href={`/models/${props.modelId}/`}
						>
							{props.repoTitle}
						</ui.Link>
					</div>
				</div>
				<div class={modelGridClass} style={{ gridArea: 'model' }}>
					<div class={modelTopbarClass} style={{ gridArea: 'model-topbar' }}>
						<div>
							<ui.Details
								options={
									props.repoModels.map(model => ({
										href: `/models/${model.id}/`,
										name: model.title,
									})) ?? []
								}
								summary={selectedModel}
							/>
						</div>
						<ui.Button
							download={`${props.modelTitle}.tangram`}
							href={`/api/models/${props.modelId}/download`}
						>
							Download Model
						</ui.Button>
						<ui.Button href={`/repos/${props.repoId}/models/new`}>
							Upload new model version
						</ui.Button>
					</div>
					<div style={{ gridArea: 'side-nav' }}>
						<ModelSideNav
							id={props.modelId}
							pagename={props.pagename}
							title={props.modelTitle}
						/>
					</div>
					<div style={{ gridArea: 'content' }}>
						<div class={innerClass}>{props.children}</div>
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
				<ui.SideNavItem href={`/models/${props.id}/`}>Overview</ui.SideNavItem>
				<ui.SideNavItem
					href={`/models/${props.id}/training_stats/`}
					selected={props.pagename.startsWith(
						`/models/_modelId_/training_stats/`,
					)}
				>
					Training Stats
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/models/${props.id}/training_metrics/`}
					selected={props.pagename.startsWith(
						`/models/_modelId_/training_metrics/`,
					)}
				>
					Training Metrics
				</ui.SideNavItem>
				<ui.SideNavItem href={`/models/${props.id}/introspect`}>
					Introspect
				</ui.SideNavItem>
				<ui.SideNavItem href={`/models/${props.id}/predict`}>
					Predict
				</ui.SideNavItem>
				<ui.SideNavItem href={`/models/${props.id}/tuning`}>
					Tuning
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/models/${props.id}/production_stats/`}
					selected={props.pagename.startsWith(
						`/models/_modelId_/production_stats/`,
					)}
				>
					Production Stats
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/models/${props.id}/production_metrics/`}
					selected={props.pagename.startsWith(
						`/models/_modelId_/production_metrics/`,
					)}
				>
					Production Metrics
				</ui.SideNavItem>
			</ui.SideNavSection>
		</ui.SideNav>
	)
}
