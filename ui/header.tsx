import { Children, h } from './deps'

type HeaderProps = { children?: Children }

export function Header(props: HeaderProps) {
	return <div class="header">{props.children}</div>
}
