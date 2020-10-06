import * as cookies from 'common/cookies'

// Set the tangram-timezone cookie.
let timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone
cookies.set({
	key: 'tangram-timezone',
	maxAge: 31536000, // This is one year.
	path: '/',
	sameSite: 'none',
	secure: true,
	value: timeZone,
})
