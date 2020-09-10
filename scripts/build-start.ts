let githubUsername = Deno.env.get('GITHUB_USERNAME')
let githubToken = Deno.env.get('GITHUB_TOKEN')
let Authorization = `Basic ${btoa(`${githubUsername}:${githubToken}`)}`

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
