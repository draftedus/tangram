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

ui.hydrateBoxChart('number_intervals')
ui.hydrateBoxChart('number_overall')

ui.hydrateBarChart('enum_intervals')
ui.hydrateBarChart('enum_overall')

ui.hydrateBarChart('text_overall')
