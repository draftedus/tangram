import * as cookies from 'common/cookies'

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
