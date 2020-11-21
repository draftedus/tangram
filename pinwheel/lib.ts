import { ComponentType, createElement, hydrate } from "preact"

export type PageInfo = {
	clientJsSrc?: string
	clientWasmJsSrc?: string
	cssSrcs?: string[]
	jsSrcs?: string[]
}

export type ClientProps<T> = {
	component: ComponentType<T>
	id: string
	props: T
}

export function Client<T>(props: ClientProps<T>) {
	return createElement(
		"div",
		{ id: props.id, "data-props": JSON.stringify(props.props) },
		createElement(props.component, props.props),
	)
}

export function hydrateComponent(id: string, component: ComponentType<any>) {
	let root = document.getElementById(id)
	if (!root) throw Error()
	let props = JSON.parse(root.dataset.props ?? "{}")
	hydrate(createElement(component, props), root)
}
