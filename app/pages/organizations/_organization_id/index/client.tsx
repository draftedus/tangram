import { ui } from 'deps'
// ui.toaster.show({
// 	backgroundColor: 'var(--blue)',
// 	message: `🎉 You are now on the ${plan} plan!`,
// })

let onAddPaymentMethod = async () => {
	// get the stripe publishable key
	let stripeUrl = 'https://js.stripe.com/v3/'
	let stripeScript: HTMLElement | null = document.querySelector(
		`script[src="${stripeUrl}"]`,
	)
	let stripePublishableKey = stripeScript?.dataset.publishableKey
	var stripe = Stripe(stripePublishableKey)
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
