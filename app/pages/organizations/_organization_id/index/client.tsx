import * as ui from '@tangramhq/ui'

let onAddPaymentMethod = async () => {
	// get the stripe publishable key
	let stripe = await loadStripe()
	let response = await fetch('', {
		body: 'action=start_stripe_checkout',
		method: 'POST',
	})
	if (!response.ok) {
		throw Error()
	}
	let { stripeCheckoutSessionId } = await response.json()
	await stripe.redirectToCheckout({ sessionId: stripeCheckoutSessionId })
}

document
	.getElementById('set-payment-method-button')
	?.addEventListener('click', onAddPaymentMethod)

let params = new URLSearchParams(window.location.search)
let stripeCheckoutSessionId = params.get('session_id')
if (stripeCheckoutSessionId) {
	finishStripeCheckout(stripeCheckoutSessionId)
}

async function loadStripe(): Promise<any> {
	let stripeUrl = 'https://js.stripe.com/v3/'
	let existingTag = document.querySelector(`script[src="${stripeUrl}"]`)
	if (!existingTag) {
		let scriptTag = document.createElement('script')
		scriptTag.src = stripeUrl
		document.head.appendChild(scriptTag)
		await new Promise((resolve, reject) => {
			scriptTag.addEventListener('load', () => {
				if (Stripe) {
					resolve(Stripe)
				} else {
					reject(new Error('stripe.js not available'))
				}
			})
			scriptTag.addEventListener('error', () => {
				reject(new Error('stripe.js failed to load '))
			})
		})
	}
	let stripePublishableKey = document.getElementById('stripe-publishable-key')
		?.dataset.stripePublishableKey
	console.log({ stripePublishableKey })
	return Stripe(stripePublishableKey)
}

async function finishStripeCheckout(stripeCheckoutSessionId: string) {
	try {
		ui.progress.start()
		let response = await fetch('', {
			body: `action=finish_stripe_checkout&stripeCheckoutSessionId=${stripeCheckoutSessionId}`,
			method: 'POST',
		})
		if (!response.ok) throw Error()
		await response.text()
	} catch {
		ui.toaster.showError('an error occurred')
	} finally {
		ui.progress.stop()
		window.location.replace(location.pathname)
	}
}
