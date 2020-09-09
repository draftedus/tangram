import { ComponentChildren, h } from './deps'

type HeaderProps = { children?: ComponentChildren }

export function Header(props: HeaderProps) {
	return <div class="header">{props.children}</div>
}
