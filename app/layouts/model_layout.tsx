import './model_layout.css'
import { Layout } from './topbar_layout'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { ComponentChildren, Fragment, h } from 'preact'

type ModelLayoutProps = {
	children?: ComponentChildren
	info: ModelLayoutInfo
	pinwheelInfo: PinwheelInfo
	selectedItem: ModelSideNavItem
}

export type ModelLayoutInfo = {
	id: string
	modelId: string
	modelTitle: string
	modelVersionIds: string[]
	owner?: Owner
	title: string
}

type Owner = {
	name: string
	url: string
}

export enum ModelSideNavItem {
	Overview = 'overview',
	TrainingStats = 'training_stats',
	TrainingMetrics = 'training_metrics',
	TrainingImportances = 'training_importances',
	Prediction = 'prediction',
	Tuning = 'tuning',
	ProductionPredictions = 'production_predictions',
	ProductionStats = 'production_stats',
	ProductionMetrics = 'production_metrics',
}

export function ModelLayout(props: ModelLayoutProps) {
	let selectedModelVersionId = props.info.modelVersionIds.find(
		modelVersionId => modelVersionId == props.info.modelId,
	)
	return (
		<Layout pinwheelInfo={props.pinwheelInfo}>
			<div class="model-layout">
				<div class="model-layout-topbar">
					<div class="model-layout-owner-slash-repo-slash-model-wrapper">
						<div class="model-layout-owner-slash-repo-wrapper">
							{props.info.owner && (
								<>
									<a
										class="model-layout-owner-slash-repo-link"
										href={props.info.owner.url}
										title="owner"
									>
										{props.info.owner.name}
									</a>
									<span class="model-layout-owner-slash-repo-slash">{'/'}</span>
								</>
							)}
							<a
								class="model-layout-owner-slash-repo-link"
								href={`/repos/${props.info.id}/`}
								title="repo"
							>
								{props.info.title}
							</a>
						</div>
					</div>
					<div class="model-layout-topbar-actions-wrapper">
						<div class="model-layout-topbar-version-select-wrapper">
							{'Version:'}
							<ui.Details
								options={
									props.info.modelVersionIds.map(modelVersionId => ({
										href: `/repos/${props.info.id}/models/${modelVersionId}/`,
										name: modelVersionId,
									})) ?? []
								}
								summary={selectedModelVersionId ?? null}
							/>
						</div>
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
				<div class="model-layout-grid">
					<div class="model-layout-side-nav-wrapper">
						<ModelSideNav
							id={props.info.modelId}
							selectedItem={props.selectedItem}
							title={props.info.modelTitle}
						/>
					</div>
					<div class="model-layout-content">{props.children}</div>
				</div>
			</div>
		</Layout>
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
					href={`/repos/${props.id}/models/${props.id}/training_importances`}
					selected={props.selectedItem === ModelSideNavItem.TrainingImportances}
				>
					{'Training Importances'}
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
					href={`/repos/${props.id}/models/${props.id}/production_predictions/`}
					selected={
						props.selectedItem === ModelSideNavItem.ProductionPredictions
					}
				>
					{'Production Predictions'}
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
