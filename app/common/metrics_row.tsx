import { Children, h } from 'deps'

type MetricsRowProps = {
	children?: Children
}

export function MetricsRow(props: MetricsRowProps) {
	return <div class="metrics-row">{props.children}</div>
}
