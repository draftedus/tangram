export let chartConfig = {
	axisWidth: 2,
	barGap: 2,
	barGroupGap: 4,
	barStrokeWidth: 2,
	bottomPadding: 8,
	font: '10px sans-serif',
	fontSize: 10,
	labelPadding: 8,
	leftPadding: 8,
	maxCornerRadius: 8,
	pointHaloRadius: 8,
	pointRadius: 4,
	rightPadding: 8,
	shapArrowDepth: 4,
	shapBarGap: 10,
	shapGroupGap: 20,
	shapGroupHeight: 100,
	splineTension: 0,
	tooltipBorderRadius: 4,
	tooltipPadding: 4,
	tooltipShadowBlur: 2,
	tooltipTargetRadius: 5,
	topPadding: 8,
}

export type ChartColors = {
	axisColor: string
	borderColor: string
	crosshairsColor: string
	gridLineColor: string
	labelColor: string
	textColor: string
	titleColor: string
	tooltipBackgroundColor: string
	tooltipShadowColor: string
}

export let lightChartColors: ChartColors = {
	axisColor: '#BBBBBB',
	borderColor: '#EEEEEE',
	crosshairsColor: '#666666',
	gridLineColor: '#EEEEEE',
	labelColor: '#666666',
	textColor: '#222222',
	titleColor: '#222222',
	tooltipBackgroundColor: '#FFFFFF',
	tooltipShadowColor: 'rgba(0,0,0,.1)',
}

export let darkChartColors: ChartColors = {
	axisColor: '#AAAAAA',
	borderColor: '#333333',
	crosshairsColor: '#AAAAAA',
	gridLineColor: '#222222',
	labelColor: '#888888',
	textColor: '#EEEEEE',
	titleColor: '#EEEEEE',
	tooltipBackgroundColor: '#333333',
	tooltipShadowColor: 'rgba(0,0,0,.1)',
}

export let chartColors: { current: ChartColors } = { current: lightChartColors }
