import * as ui from "@tangramhq/ui"
import { Logo, LogoScheme } from "@tangramhq/www"
import { h } from "preact"

type TopbarProps = {
	topbarAvatar: TopbarAvatar | null
}

export type TopbarAvatar = {
	avatarUrl: string | null
}

export function Topbar(props: TopbarProps) {
	let items = []
	if (props.topbarAvatar) {
		items.push({
			element: (
				<ui.Link href="/user">
					<ui.Avatar src={props.topbarAvatar.avatarUrl} />
				</ui.Link>
			),
			href: "/user",
			title: "Settings",
		})
	}
	return (
		<ui.Topbar
			backgroundColor="var(--header-color)"
			dropdownBackgroundColor="var(--surface-color)"
			foregroundColor="var(--text-color)"
			items={items}
			logo={<Logo colorScheme={LogoScheme.Multi} />}
			logoHref="/"
			title="tangram"
		/>
	)
}
