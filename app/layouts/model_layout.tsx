import { TopbarLayout } from './topbar_layout'
import { Children, PinwheelInfo, h, ui } from 'deps'

type ModelLayoutProps = {
	children?: Children
	info: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	selectedItem: ModelSideNavItem
}

export type ModelLayoutInfo = {
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
	title: string
}

export enum ModelSideNavItem {
	Overview = 'overview',
	TrainingStats = 'training_stats',
	TrainingMetrics = 'training_metrics',
	Introspection = 'introspection',
	Prediction = 'prediction',
	Tuning = 'tuning',
	ProductionStats = 'production_stats',
	ProductionMetrics = 'production_metrics',
}

export function ModelLayout(props: ModelLayoutProps) {
	let selectedModel = props.info.models.find(
		model => model.id == props.info.modelId,
	)
	return (
		<TopbarLayout pinwheelInfo={props.pinwheelInfo}>
			<div class="model-layout">
				<div class="model-layout-topbar" style={{ gridArea: 'topbar' }}>
					<div class="model-layout-owner-slash-repo-slash-model-wrapper">
						<div class="model-layout-owner-slash-repo-wrapper">
							<a
								class="model-layout-owner-slash-repo-link"
								href={props.info.ownerUrl}
								title="owner"
							>
								{props.info.ownerName}
							</a>
							<span class="model-layout-owner-slash-repo-slash">{'/'}</span>
							<a
								class="model-layout-owner-slash-repo-link"
								href={`/repos/${props.info.id}/models/${props.info.modelId}/`}
								title="repo"
							>
								{props.info.title}
							</a>
						</div>
					</div>
					<div class="model-layout-topbar-actions-wrapper">
						{'Version:'}
						<ui.Details
							options={
								props.info.models.map(model => ({
									href: `/repos/${props.info.id}/models/${model.id}/`,
									name: model.id,
								})) ?? []
							}
							summary={selectedModel?.id ?? null}
						/>
						<ui.Button
							download={`${props.info.modelTitle}.tangram`}
							href={`/repos/${props.info.id}/models/${props.info.modelId}/download`}
						>
							{'Download'}
						</ui.Button>
						<ui.Button href={`/repos/${props.info.id}/models/new`}>
							{'Upload New Version'}
						</ui.Button>
					</div>
				</div>
				<div class="model-layout-grid" style={{ gridArea: 'model' }}>
					<div
						class="model-layout-sidenav-wrapper"
						style={{ gridArea: 'side-nav' }}
					>
						<ModelSideNav
							id={props.info.modelId}
							selectedItem={props.selectedItem}
							title={props.info.modelTitle}
						/>
					</div>
					<div class="model-layout-content" style={{ gridArea: 'content' }}>
						{props.children}
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
					selected={props.selectedItem === ModelSideNavItem.TrainingStats}
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
					href={`/repos/${props.id}/models/${props.id}/prediction`}
					selected={props.selectedItem === ModelSideNavItem.Prediction}
				>
					{'Prediction'}
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
