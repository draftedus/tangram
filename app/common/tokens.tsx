import * as ui from "@tangramhq/ui"
import { h } from "preact"

export let trainingColor = ui.colors.blue
export let productionColor = ui.colors.green
export let baselineColor = ui.colors.gray
export let selectedThresholdColor = ui.colors.blue

export function UnknownColumnToken() {
	return <ui.Token color="var(--gray)">{"Unknown"}</ui.Token>
}

export function NumberColumnToken() {
	return <ui.Token color="var(--teal)">{"Number"}</ui.Token>
}

export function EnumColumnToken() {
	return <ui.Token color="var(--purple)">{"Enum"}</ui.Token>
}

export function TextColumnToken() {
	return <ui.Token color="var(--orange)">{"Text"}</ui.Token>
}
