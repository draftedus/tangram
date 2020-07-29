let Authorization = `Basic ${btoa(
	`${Deno.env.get('GITHUB_USERNAME')}:${Deno.env.get('GITHUB_TOKEN')}`,
)}`

let response = await fetch(
	'https://api.github.com/repos/tangram-hq/tangram/dispatches',
	{
		body: JSON.stringify({ ['event_type']: 'build' }),
		headers: { Authorization },
		method: 'POST',
	},
)

if (!response.ok) {
	throw Error(response.statusText)
}
