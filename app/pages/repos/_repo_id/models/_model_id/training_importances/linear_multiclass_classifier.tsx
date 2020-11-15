import { FeatureImportancesTable } from "./feature_importances_table"
import { LinearMulticlassClassifierProps } from "./props"
import * as ui from "@tangramhq/ui"
import { h } from "preact"

export function LinearMulticlassClassifierTrainingImportancesPage(
	props: LinearMulticlassClassifierProps,
) {
	return (
		<ui.S1>
			<ui.H1>{"Training Feature Importances"}</ui.H1>
			<ui.P>
				{"Your model had a total of "}
				{props.featureImportances.length}
				{" features."}
			</ui.P>
			<FeatureImportancesTable featureImportances={props.featureImportances} />
		</ui.S1>
	)
}
