import { ui } from 'deps'

let selectElements = document.querySelectorAll('select')
selectElements.forEach(selectElement => {
	selectElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest('form')
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
})

ui.hydrateBarChart('prediction_count')

if (
	document.getElementById('quantiles_overall') &&
	document.getElementById('quantiles_intervals')
) {
	ui.hydrateBoxChart('quantiles_overall')
	ui.hydrateBoxChart('quantiles_intervals')
}

if (
	document.getElementById('histogram_overall') &&
	document.getElementById('histogram_intervals')
) {
	ui.hydrateBarChart('histogram_overall')
	ui.hydrateBarChart('histogram_intervals')
}
