import {
	hydrateBarChart,
	hydrateBoxChart,
	hydrateFeatureContributionsChart,
} from "@tangramhq/charts"

let inputFieldBarCharts = document.querySelectorAll(
	'.column-chart[data-chart-type="bar"]',
)
inputFieldBarCharts.forEach(barChart => {
	hydrateBarChart(barChart.id)
})

let inputFieldBoxCharts = document.querySelectorAll(
	'.column-chart[data-chart-type="box"]',
)
inputFieldBoxCharts.forEach(boxChart => {
	hydrateBoxChart(boxChart.id)
})

if (document.getElementById("probabilities")) {
	hydrateBarChart("probabilities")
}

if (document.getElementById("regression_feature_contributions")) {
	hydrateFeatureContributionsChart("regression_feature_contributions")
}
if (document.getElementById("binary_classification_feature_contributions")) {
	hydrateFeatureContributionsChart(
		"binary_classification_feature_contributions",
	)
}
if (
	document.getElementById("multiclass_classification_feature_contributions")
) {
	hydrateFeatureContributionsChart(
		"multiclass_classification_feature_contributions",
	)
}
