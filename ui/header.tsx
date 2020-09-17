import './header.css'
import { ComponentChildren, h } from 'preact'

type HeaderProps = { children?: ComponentChildren }

export function Header(props: HeaderProps) {
	return <div class="header">{props.children}</div>
}
