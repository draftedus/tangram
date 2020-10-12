import {
	hydrateBarChart,
	hydrateBoxChart,
	hydrateFeatureContributionsChart,
} from '@tangramhq/charts'

if (document.getElementById('probabilities')) {
	hydrateBarChart('probabilities')
}
if (document.getElementById('regression_feature_contributions')) {
	hydrateFeatureContributionsChart('regression_feature_contributions')
}
if (document.getElementById('classification_feature_contributions')) {
	hydrateFeatureContributionsChart('classification_feature_contributions')
}

let barCharts = document.querySelectorAll(
	'.column-chart[data-chart-type="bar"]',
)
barCharts.forEach(barChart => {
	hydrateBarChart(barChart.id)
})

let boxCharts = document.querySelectorAll(
	'.column-chart[data-chart-type="box"]',
)
boxCharts.forEach(boxChart => {
	hydrateBoxChart(boxChart.id)
})
