import { ui } from 'deps'

let classSelectElements = document.querySelectorAll('#class-select')
classSelectElements.forEach(selectElement => {
	if (!(selectElement instanceof HTMLSelectElement)) throw Error()
	selectElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest('form')
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
})

let dateWindowSlectElements = document.querySelectorAll('#date-window-select')
dateWindowSlectElements.forEach(selectElement => {
	if (!(selectElement instanceof HTMLSelectElement)) throw Error()
	selectElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest('form')
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
})

ui.hydrateLineChart('precision_intervals')
ui.hydrateLineChart('recall_intervals')
ui.hydrateLineChart('f1_intervals')
