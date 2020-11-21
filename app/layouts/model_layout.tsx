import { AppLayoutInfo } from "./app_layout"
import "./model_layout.css"
import { PageInfo } from "@tangramhq/pinwheel"
import * as ui from "@tangramhq/ui"
import { Document } from "@tangramhq/www"
import { Topbar, TopbarAvatar } from "common/topbar"
import { ComponentChildren, Fragment, h } from "preact"

type ModelLayoutProps = {
	children?: ComponentChildren
	info: ModelLayoutInfo
	pageInfo: PageInfo
	selectedItem: ModelSideNavItem
}

export type ModelLayoutInfo = {
	appLayoutInfo: AppLayoutInfo
	modelId: string
	modelVersionIds: string[]
	owner: Owner | null
	repoId: string
	repoTitle: string
	topbarAvatar: TopbarAvatar | null
}

type Owner =
	| { type: "user"; value: UserOwner }
	| { type: "organization"; value: OrganizationOwner }

type UserOwner = {
	email: string
	id: string
}

type OrganizationOwner = {
	id: string
	name: string
}

export enum ModelSideNavItem {
	Overview = "overview",
	TrainingGrid = "training_grid",
	TrainingStats = "training_stats",
	TrainingMetrics = "training_metrics",
	TrainingImportances = "training_importances",
	Prediction = "prediction",
	Tuning = "tuning",
	ProductionPredictions = "production_predictions",
	ProductionStats = "production_stats",
	ProductionMetrics = "production_metrics",
}

export function ModelLayout(props: ModelLayoutProps) {
	let selectedModelVersionId = props.info.modelVersionIds.find(
		modelVersionId => modelVersionId == props.info.modelId,
	)
	if (!selectedModelVersionId) throw Error()
	return (
		<Document pageInfo={props.pageInfo}>
			<div class="model-layout-topbar-grid">
				<Topbar topbarAvatar={props.info.topbarAvatar} />
				<div class="model-layout">
					<ModelLayoutTopbar
						modelLayoutInfo={props.info}
						selectedModelVersionId={selectedModelVersionId}
					/>
					<div class="model-layout-grid">
						<div class="model-layout-side-nav-wrapper">
							<ModelSideNav
								id={props.info.modelId}
								repoTitle={props.info.repoTitle}
								selectedItem={props.selectedItem}
							/>
						</div>
						<div class="model-layout-content">{props.children}</div>
					</div>
				</div>
			</div>
		</Document>
	)
}

type ModelLayoutTopbarProps = {
	modelLayoutInfo: ModelLayoutInfo
	selectedModelVersionId: string
}

function ModelLayoutTopbar(props: ModelLayoutTopbarProps) {
	let owner = props.modelLayoutInfo.owner
	let ownerTitle
	let ownerUrl
	if (owner?.type === "user") {
		ownerTitle = owner.value.email
		ownerUrl = `/user`
	} else if (owner?.type === "organization") {
		ownerTitle = owner.value.name
		ownerUrl = `/organizations/${owner.value.id}`
	}
	return (
		<div class="model-layout-topbar">
			<div class="model-layout-owner-slash-repo-slash-model-wrapper">
				<div class="model-layout-owner-slash-repo-wrapper">
					{props.modelLayoutInfo.owner && (
						<>
							<a
								class="model-layout-owner-slash-repo-link"
								href={ownerUrl}
								title="owner"
							>
								{ownerTitle}
							</a>
							<span class="model-layout-owner-slash-repo-slash">{"/"}</span>
						</>
					)}
					<a
						class="model-layout-owner-slash-repo-link"
						href={`/repos/${props.modelLayoutInfo.repoId}/`}
						title="repo"
					>
						{props.modelLayoutInfo.repoTitle}
					</a>
				</div>
			</div>
			<div class="model-layout-topbar-actions-wrapper">
				<div class="model-layout-topbar-version-select-wrapper">
					<ui.Details
						options={
							props.modelLayoutInfo.modelVersionIds.map(modelVersionId => ({
								href: `/repos/${props.modelLayoutInfo.repoId}/models/${modelVersionId}/`,
								title: modelVersionId,
							})) ?? []
						}
						summary={`Version: ${props.selectedModelVersionId}`}
					/>
				</div>
				<ui.Button
					download={`${props.modelLayoutInfo.repoTitle}.tangram`}
					href={`/repos/${props.modelLayoutInfo.repoId}/models/${props.modelLayoutInfo.modelId}/download`}
				>
					{"Download"}
				</ui.Button>
				<ui.Button href={`/repos/${props.modelLayoutInfo.repoId}/models/new`}>
					{"Upload New Version"}
				</ui.Button>
			</div>
		</div>
	)
}

type ModelSideNavProps = {
	id: string
	repoTitle: string
	selectedItem: ModelSideNavItem
}

function ModelSideNav(props: ModelSideNavProps) {
	return (
		<ui.SideNav>
			<ui.SideNavSection>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/`}
					selected={props.selectedItem === ModelSideNavItem.Overview}
				>
					{"Overview"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_grid/`}
					selected={props.selectedItem === ModelSideNavItem.TrainingGrid}
				>
					{"Training Grid"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_stats/`}
					selected={props.selectedItem === ModelSideNavItem.TrainingStats}
				>
					{"Training Stats"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_metrics/`}
					selected={props.selectedItem === ModelSideNavItem.TrainingMetrics}
				>
					{"Training Metrics"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/training_importances`}
					selected={props.selectedItem === ModelSideNavItem.TrainingImportances}
				>
					{"Training Importances"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/prediction`}
					selected={props.selectedItem === ModelSideNavItem.Prediction}
				>
					{"Prediction"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/tuning`}
					selected={props.selectedItem === ModelSideNavItem.Tuning}
				>
					{"Tuning"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_predictions/`}
					selected={
						props.selectedItem === ModelSideNavItem.ProductionPredictions
					}
				>
					{"Production Predictions"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_stats/`}
					selected={props.selectedItem === ModelSideNavItem.ProductionStats}
				>
					{"Production Stats"}
				</ui.SideNavItem>
				<ui.SideNavItem
					href={`/repos/${props.id}/models/${props.id}/production_metrics/`}
					selected={props.selectedItem === ModelSideNavItem.ProductionMetrics}
				>
					{"Production Metrics"}
				</ui.SideNavItem>
			</ui.SideNavSection>
		</ui.SideNav>
	)
}
