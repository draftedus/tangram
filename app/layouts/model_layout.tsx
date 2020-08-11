import { TopbarLayout } from './topbar_layout'
import { Children, PinwheelInfo, h, r, ui } from 'deps'

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
	pinwheelInfo: PinwheelInfo
	selectedItem: ModelSideNavItem
	title: string
}

enum ModelSideNavItem {
	Overview = 'overview',
	TrainingStats = 'training_stats',
	TrainingMetrics = 'training_metrics',
	Introspection = 'introspection',
	Predict = 'precict',
	Tuning = 'tuning',
	ProductionStats = 'production_stats',
	ProductionMetrics = 'production_metrics',
}

export function ModelLayout(props: ModelLayoutProps) {
	let selectedModel = r(props.models.find(model => model.id == props.modelId))
		?.title
	return (
		<TopbarLayout pinwheelInfo={props.pinwheelInfo}>
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
							selectedItem={props.selectedItem}
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
	selectedItem: ModelSideNavItem
	title: string
}

function ModelSideNav(props: ModelSideNavProps) {
	return (
		<ui.SideNav>
			<ui.SideNavSection>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/`}
					selected={props.selectedItem === ModelSideNavItem.Overview}
				>
					{'Overview'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_stats/`}
					selected={props.selectedItem === ModelSideNavItem.Overview}
				>
					{'Training Stats'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_metrics/`}
					selected={props.selectedItem === ModelSideNavItem.TrainingMetrics}
				>
					{'Training Metrics'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/introspection`}
					selected={props.selectedItem === ModelSideNavItem.Introspection}
				>
					{'Introspection'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/predict`}
					selected={props.selectedItem === ModelSideNavItem.Predict}
				>
					{'Predict'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/tuning`}
					selected={props.selectedItem === ModelSideNavItem.Tuning}
				>
					{'Tuning'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_stats/`}
					selected={props.selectedItem === ModelSideNavItem.ProductionStats}
				>
					{'Production Stats'}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_metrics/`}
					selected={props.selectedItem === ModelSideNavItem.ProductionMetrics}
				>
					{'Production Metrics'}
				</ui.SideNavItem>
			</ui.SideNavSection>
		</ui.SideNav>
	)
}
