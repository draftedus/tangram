import { ui } from 'deps'

if (document.getElementById('probabilities')) {
	ui.hydrateBarChart('probabilities')
}
if (document.getElementById('regression_shap')) {
	ui.hydrateShapChart('regression_shap')
}
if (document.getElementById('classification_shap')) {
	ui.hydrateShapChart('classification_shap')
}

let barCharts = document.querySelectorAll(
	'.column-chart[data-chart-type="bar"]',
)
barCharts.forEach(barChart => {
	ui.hydrateBarChart(barChart.id)
})

let boxCharts = document.querySelectorAll(
	'.column-chart[data-chart-type="box"]',
)
boxCharts.forEach(boxChart => {
	ui.hydrateBoxChart(boxChart.id)
})
