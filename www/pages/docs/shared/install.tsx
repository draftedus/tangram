import * as ui from '@tangramhq/ui'
import { Fragment, h } from 'preact'

export function Install() {
	return (
		<>
			<ui.H2>{'Brew'}</ui.H2>
			<Brew />
			<ui.H2>{'Scoop'}</ui.H2>
			<Scoop />
			<ui.H2>{'MacOS'}</ui.H2>
			<MacOS />
			<ui.H2>{'Linux'}</ui.H2>
			<Linux />
			<ui.H2>{'Windows'}</ui.H2>
			<Windows />
			<ui.H2>{'Docker'}</ui.H2>
			<Docker />
		</>
	)
}

function Brew() {
	let code = `$ brew install tangram-hq/tap/tangram`
	return <ui.Code code={code} hideLineNumbers={true} />
}

function Scoop() {
	let code = `$ scoop bucket add tangram https://github.com/tangram-hq/scoop.git\n$ scoop install tangram`
	return <ui.Code code={code} hideLineNumbers={true} />
}

function MacOS() {
	return (
		<ui.OrderedList>
			<ui.ListItem>
				{
					'Download the executable for macOS from https://github.com/tangram-hq/tangram/releases/latest.'
				}
			</ui.ListItem>
			<ui.ListItem>
				{'Unzip it: '}
				<ui.InlineCode>
					{'tar -xvf tangram-cli-X.X.X-macos-amd64.tar.gz'}
				</ui.InlineCode>
			</ui.ListItem>
			<ui.ListItem>{'Run the unzipped executable.'}</ui.ListItem>
			<ui.ListItem>
				{
					'You will probably want to move the executable somewhere in your `$PATH` such as `/usr/local/bin`.'
				}
			</ui.ListItem>
		</ui.OrderedList>
	)
}

function Linux() {
	return (
		<ui.OrderedList>
			<ui.ListItem>
				{
					'Download the executable for Linux from https://github.com/tangram-hq/tangram/releases/latest.'
				}
			</ui.ListItem>
			<ui.ListItem>
				{'Unzip it: '}
				<ui.InlineCode>
					{'tar -xvf tangram-cli-X.X.X-linux-amd64.tar.gz'}
				</ui.InlineCode>
			</ui.ListItem>
			<ui.ListItem>{'Run the unzipped executable.'}</ui.ListItem>
			<ui.ListItem>
				{
					'You will probably want to move the executable somewhere in your `$PATH` such as `/usr/local/bin`.'
				}
			</ui.ListItem>
		</ui.OrderedList>
	)
}

function Windows() {
	return (
		<ui.OrderedList>
			<ui.ListItem>
				{
					'Download the executable for Windows from https://github.com/tangram-hq/tangram/releases/latest.'
				}
			</ui.ListItem>
			<ui.ListItem>{'Unzip it.'}</ui.ListItem>
			<ui.ListItem>{'Run the unzipped executable.'}</ui.ListItem>
		</ui.OrderedList>
	)
}

function Docker() {
	let code = `$ docker run --rm -it tangramhq/tangram train --file heart-disease.csv --target diagnosis`
	return <ui.Code code={code} hideLineNumbers={true} />
}
