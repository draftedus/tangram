import * as cookies from './cookies'
import { ui } from './deps'

// set the tangram-timezone cookie
let timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone
cookies.set({
	key: 'tangram-timezone',
	maxAge: 31536000, // one year
	path: '/',
	sameSite: 'none',
	secure: true,
	value: timeZone,
})

// set chart font
ui.chartConfig.font = '14px JetBrains Mono'
ui.chartConfig.fontSize = 14
