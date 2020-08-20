export * from '@tangramhq/pinwheel'
export * as ui from '@tangramhq/ui'
export * from 'preact/hooks'
export * from 'preact-render-to-string'
export {
	h,
	createElement,
	render,
	hydrate,
	Component,
	Fragment,
	cloneElement,
	createContext,
	createRef,
	isValidElement,
	options,
	toChildArray,
} from 'preact'
export type {
	AnyComponent,
	Attributes,
	ClassAttributes,
	ComponentChild,
	ComponentChildren,
	ComponentChildren as Children,
	ComponentClass,
	ComponentConstructor,
	ComponentFactory,
	ComponentProps,
	ComponentType,
	Consumer,
	Context,
	FunctionComponent,
	FunctionalComponent,
	JSX,
	Key,
	Options,
	PreactConsumer,
	PreactContext,
	PreactDOMAttributes,
	PreactProvider,
	Provider,
	Ref,
	RefCallback,
	RefObject,
	RenderableProps,
	VNode,
} from 'preact'

import { JSX } from 'preact'
import { renderToString } from 'preact-render-to-string'

export function renderPage(element: JSX.Element): string {
	return '<!doctype html>' + renderToString(element)
}
