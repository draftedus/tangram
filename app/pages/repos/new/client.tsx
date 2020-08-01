export {}

let fileInputElements = document.querySelectorAll('input[type=file]')
fileInputElements.forEach(fileInputElement => {
	fileInputElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLInputElement)) throw Error()
		let file = event.currentTarget.files?.item(0)
		if (file) {
			event.currentTarget.parentElement?.firstChild?.replaceWith(
				document.createTextNode(file.name),
			)
		}
	})
})
