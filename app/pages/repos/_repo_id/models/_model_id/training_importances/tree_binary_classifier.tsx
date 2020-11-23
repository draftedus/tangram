import { FeatureImportancesTable } from "./feature_importances_table"
import { TreeBinaryClassifierProps } from "./props"
import * as ui from "@tangramhq/ui"
import { h } from "preact"

export function TreeBinaryClassifierTrainingImportancesPage(
	props: TreeBinaryClassifierProps,
) {
	return (
		<ui.S1>
			<ui.H1>{"Training Feature Importances"}</ui.H1>
			<ui.P>
				{"Your model had a total of "}
				{props.nFeatures}
				{" features."}
			</ui.P>
			<FeatureImportancesTable featureImportances={props.featureImportances} />
		</ui.S1>
	)
}
