import {
	hydrateBarChart,
	hydrateBoxChart,
	hydrateShapChart,
} from '@tangramhq/charts'

if (document.getElementById('probabilities')) {
	hydrateBarChart('probabilities')
}
if (document.getElementById('regression_shap')) {
	hydrateShapChart('regression_shap')
}
if (document.getElementById('classification_shap')) {
	hydrateShapChart('classification_shap')
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
