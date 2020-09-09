import { ComponentChildren, h } from 'deps'

type MetricsRowProps = {
	children?: ComponentChildren
}

export function MetricsRow(props: MetricsRowProps) {
	return <div class="metrics-row">{props.children}</div>
}
