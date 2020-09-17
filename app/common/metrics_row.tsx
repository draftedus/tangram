import './metrics_row.css'
import { ComponentChildren, h } from 'preact'

type MetricsRowProps = {
	children?: ComponentChildren
}

export function MetricsRow(props: MetricsRowProps) {
	return <div class="metrics-row">{props.children}</div>
}
