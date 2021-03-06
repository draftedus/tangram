import "./asciicast.css"
import { h } from "preact"
import { useEffect, useRef } from "preact/hooks"

type AsciicastProps = {
	asciicast: AsciicastEvent[]
	height: string
	repeat?: boolean
	repeatDelay?: number
}

export type AsciicastEvent = [number, string]

export function Asciicast(props: AsciicastProps) {
	let term = useRef<AsciicastPlayer | null>(null)
	let termRef = useRef<HTMLDivElement | null>(null)

	useEffect(() => {
		if (!term.current) {
			if (!termRef.current) {
				throw Error()
			}
			term.current = new AsciicastPlayer(termRef.current, props.asciicast, {
				repeat: props.repeat ?? true,
				repeatDelay: props.repeatDelay ?? 1000,
			})
			term.current.start()
		}
	}, [props.asciicast, props.repeat, props.repeatDelay])

	let style = { height: props.height }

	return <div class="asciicast" ref={termRef} style={style} />
}

type AsciicastPlayerOptions = {
	repeat: boolean
	repeatDelay: number
}

class AsciicastPlayer {
	private asciicast: AsciicastEvent[]
	private frameIndex?: number
	private options: AsciicastPlayerOptions
	private startTime?: number
	private timeout?: ReturnType<typeof setTimeout>
	private element: HTMLElement

	public constructor(
		element: HTMLElement,
		asciicast: AsciicastEvent[],
		options: AsciicastPlayerOptions,
	) {
		this.asciicast = asciicast
		this.element = element
		this.options = options
	}

	public render = () => {
		if (this.startTime === undefined || this.frameIndex === undefined) {
			throw Error()
		}
		let currentTime = performance.now() - this.startTime
		while (true) {
			if (this.frameIndex >= this.asciicast.length) {
				break
			}
			let event = this.asciicast[this.frameIndex]
			if (event == undefined) throw Error()
			let eventTime = event[0] * 1000
			if (currentTime < eventTime) {
				break
			}
			this.element.innerHTML += event[1]
			this.frameIndex++
		}
		if (this.frameIndex < this.asciicast.length) {
			let nextEvent = this.asciicast[this.frameIndex]
			if (nextEvent == undefined) throw Error()
			let nextEventTime = nextEvent[0] * 1000
			let currentTime = performance.now() - this.startTime
			this.timeout = setTimeout(
				this.render,
				Math.max(nextEventTime - currentTime, 0),
			)
		} else {
			if (this.options.repeat) {
				setTimeout(() => this.start(), this.options.repeatDelay)
			}
		}
	}

	public start = () => {
		this.element.innerHTML = ""
		this.startTime = performance.now()
		this.frameIndex = 0
		this.render()
	}

	public stop = () => {
		if (this.timeout) {
			clearTimeout(this.timeout)
		}
	}
}
