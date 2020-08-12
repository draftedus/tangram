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

let boxCharts = document.querySelectorAll('[data-chart-type="box"]')
boxCharts.forEach(boxChart => {
	if (!(boxChart instanceof HTMLDivElement)) throw Error
	ui.hydrateBoxChart(boxChart.id)
})

let barCharts = document.querySelectorAll('[data-chart-type="bar"]')
barCharts.forEach(barChart => {
	if (!(barChart instanceof HTMLDivElement)) throw Error
	ui.hydrateBarChart(barChart.id)
})

// // on change of a form element on this page, clear the predictions
// let form = document.getElementById('predict_form')
// if (!(form instanceof HTMLFormElement)) throw Error()
// for (let i = 0; i < form.elements.length; i++) {
// 	let element = form.elements[i]
// 	element.addEventListener('change', (_: Event) => {
// 		let predictOutputContainer: HTMLElement | null = document.getElementById(
// 			'predict_output',
// 		)
// 		if (predictOutputContainer) {
// 			predictOutputContainer.innerHTML = ''
// 		}
// 	})
// }
