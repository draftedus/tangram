export enum DateWindow {
	Today = 'today',
	ThisMonth = 'this_month',
	ThisYear = 'this_year',
}

export enum DateWindowInterval {
	Hourly = 'hourly',
	Daily = 'daily',
	Monthly = 'monthly',
}

export function intervalChartTitle(
	dateWindowInterval: DateWindowInterval,
	title: string,
) {
	switch (dateWindowInterval) {
		case DateWindowInterval.Hourly:
			return `Hourly ${title}`
		case DateWindowInterval.Daily:
			return `Daily ${title}`
		case DateWindowInterval.Monthly:
			return `Monthly ${title}`
	}
}

export function overallChartTitle(dateWindow: DateWindow, title: string) {
	switch (dateWindow) {
		case DateWindow.Today:
			return `Today's ${title}`
		case DateWindow.ThisMonth:
			return `This Month's ${title}`
		case DateWindow.ThisYear:
			return `This Year's ${title}`
	}
}

export function formatDateWindowInterval(
	dateString: string,
	dateWindowInterval: DateWindowInterval,
) {
	let date = new Date(dateString)
	switch (dateWindowInterval) {
		case DateWindowInterval.Hourly:
			return formatHour(date)
		case DateWindowInterval.Daily:
			return formatDayOfMonth(date)
		case DateWindowInterval.Monthly:
			return formatMonth(date)
	}
}

export function formatDateWindow(dateString: string, dateWindow: DateWindow) {
	let date = new Date(dateString)
	switch (dateWindow) {
		case DateWindow.Today:
			return formatDay(date)
		case DateWindow.ThisMonth:
			return formatMonth(date)
		case DateWindow.ThisYear:
			return formatYear(date)
	}
}

let months = [
	'Jan',
	'Feb',
	'Mar',
	'Apr',
	'May',
	'Jun',
	'Jul',
	'Aug',
	'Sep',
	'Oct',
	'Nov',
	'Dec',
]

function formatHour(date: Date): string {
	let hours12 = (hours: number) => {
		let hours12 = hours % 12
		if (hours12 == 0) {
			return 12
		}
		return hours12
	}

	let meridiem = (hours: number) => (hours < 12 || hours == 24 ? 'AM' : 'PM')

	let hours = date.getHours()

	if (hours12(hours) == 0) {
		return '12AM - 1AM'
	}
	return `${hours12(hours)}${meridiem(hours)}-${hours12(hours + 1)}${meridiem(
		hours + 1,
	)}`
}

function formatDay(date: Date): string {
	return date.toDateString()
}

function formatYear(date: Date): string {
	return date.getFullYear().toString()
}

function formatMonth(date: Date): string {
	let month = date.getMonth()
	let year = date.getFullYear()
	return `${months[month]} ${year}`
}

function formatDayOfMonth(date: Date): string {
	let month = date.getMonth()
	let day = date.getDate()
	return `${months[month]} ${day}`
}
