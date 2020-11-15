enum Status {
	Running,
	Stopping,
	Stopped,
}

type State =
	| {
			element: HTMLDivElement
			status: Status.Running
			timer: ReturnType<typeof setInterval>
			value: number
	  }
	| { status: Status.Stopping }
	| { status: Status.Stopped }

let state: State = { status: Status.Stopped }

function start() {
	if (state.status !== Status.Stopped) {
		return
	}
	let element = document.createElement("div")
	element.style.position = "fixed"
	element.style.height = "4px"
	element.style.top = "0"
	element.style.left = "0"
	element.style.right = "100%"
	element.style.background = `linear-gradient(to left, var(--purple), var(--red), var(--orange), var(--yellow), var(--green), var(--teal), var(--blue), var(--indigo))`
	element.style.transition = "right 0.2s, opacity 0.2s"
	element.style.opacity = "0"
	document.body.appendChild(element)
	let timer = setInterval(() => inc(), 400)
	state = { element, status: Status.Running, timer, value: 0 }
	state.value = 10
	state.element.style.right = "90%"
	state.element.style.opacity = "1"
}

function inc(amount?: number) {
	if (state.status !== Status.Running) {
		return
	}
	state.value = Math.min(state.value + (amount ?? 10), 90)
	state.element.style.right = `${100 - state.value}%`
}

async function stop() {
	if (state.status !== Status.Running) {
		return
	}
	let { element, timer } = state
	clearInterval(timer)
	state = { status: Status.Stopping }
	element.style.right = "0%"
	await new Promise(resolve => setTimeout(resolve, 200))
	element.style.opacity = "0"
	await new Promise(resolve => setTimeout(resolve, 200))
	document.body.removeChild(element)
	state = { status: Status.Stopped }
}

export let progress = {
	inc,
	start,
	stop,
}
