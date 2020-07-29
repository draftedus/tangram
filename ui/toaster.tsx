import { colors, variables } from './theme'

type Toast = {
	backgroundColor: string
	message: string
	textColor?: string
}

let element: HTMLDivElement | undefined
let duration = 2000
let keyframes: Keyframe[] = [
	{
		offset: 0,
		top: '-4rem',
	},
	{
		offset: 0.1,
		top: '.75rem',
	},
	{
		offset: 0.9,
		top: '.75rem',
	},
	{
		offset: 1.0,
		top: '-4rem',
	},
]

async function show(toast: Toast) {
	if (element) {
		return
	}
	element = document.createElement('div')
	element.style.zIndex = '1'
	element.style.position = 'fixed'
	element.style.top = '0'
	element.style.left = '0'
	element.style.bottom = '0'
	element.style.right = '0'
	element.style.display = 'grid'
	element.style.justifyContent = 'center'
	element.style.alignContent = 'start'
	element.style.pointerEvents = 'none'
	let inner = document.createElement('div')
	inner.style.padding = '1rem'
	inner.style.borderRadius = variables.border.radius
	inner.style.backgroundColor = toast.backgroundColor
	inner.style.color = toast.textColor ?? variables.colors.funText
	inner.appendChild(document.createTextNode(toast.message))
	element.appendChild(inner)
	document.body.appendChild(element)
	element.animate(keyframes, { duration })
	await new Promise(r => setTimeout(r, duration))
	document.body.removeChild(element)
	element = undefined
}

function showMessage(message: string) {
	show({
		backgroundColor: variables.colors.accent,
		message,
	})
}

function showError(message: string) {
	show({
		backgroundColor: colors.red,
		message,
	})
}

export let toaster = {
	show,
	showError,
	showMessage,
}
