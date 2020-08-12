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

if (
	document.getElementById('number_intervals') &&
	document.getElementById('number_overall')
) {
	ui.hydrateBoxChart('number_intervals')
	ui.hydrateBoxChart('number_overall')
}

if (
	document.getElementById('enum_intervals') &&
	document.getElementById('enum_overall')
) {
	ui.hydrateBarChart('enum_intervals')
	ui.hydrateBarChart('enum_overall')
}

if (document.getElementById('text_overall')) {
	ui.hydrateBarChart('text_overall')
}
