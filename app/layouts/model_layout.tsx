import { TopbarLayout } from './topbar_layout'
import { Children, h, r, ui } from 'deps'

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

export function ModelLayout(props: ModelLayoutProps) {
	let selectedModel = r(props.models.find(model => model.id == props.modelId))
		?.title
	return (
		<TopbarLayout>
			<div class="model-layout">
				<div
					class="model-layout-model-group-topbar"
					style={{ gridArea: 'model-group-topbar' }}
				>
					<div class="model-layout-owner-slash-repo-wrapper">
						<a
							class="model-layout-owner-slash-repo-link"
							href={props.ownerUrl}
							title="owner"
						>
							{props.ownerName}
						</a>
						<span class="model-layout-owner-slash-repo-link">{'/'}</span>
						<a
							class="model-layout-owner-slash-repo-link"
							href={`/repos/${props.id}/models/${props.modelId}/`}
							title="repo"
						>
							{props.title}
						</a>
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
						{'Upload New Model Version'}
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
