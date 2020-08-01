export {}

let selectElements = document.querySelectorAll('select')
selectElements.forEach(selectElement => {
	if (!(selectElement instanceof HTMLSelectElement)) throw Error()
	selectElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest('form')
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
})
