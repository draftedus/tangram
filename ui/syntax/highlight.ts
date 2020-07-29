import { Replacement, SyntaxColors, SyntaxGrammar } from './types'

export function highlight(
	text: string,
	grammar: SyntaxGrammar,
	colors: SyntaxColors,
): string {
	// replacements stores the range and scope
	// for each part of the text to highlight.
	let replacements: Replacement[] = []

	// for each rule in the grammar
	let match
	for (let { regex, scope } of grammar) {
		// for each match of the rule in the text
		while ((match = regex.exec(text)) !== null) {
			// this is the range of the match in the text
			let range = {
				end: match.index + match[0].length,
				start: match.index,
			}
			// only use this match if it doesn't overlap
			// with any of the existing replacements
			let foundOverlap = false
			for (let { range: existingRange } of replacements) {
				if (
					(range.start <= existingRange.start &&
						range.end >= existingRange.start) ||
					(existingRange.start <= range.start &&
						existingRange.end >= range.start)
				) {
					foundOverlap = true
					break
				}
			}
			if (!foundOverlap) {
				replacements.push({
					range,
					scope,
				})
			}
		}
	}

	// sort the replacements by start index
	replacements.sort((a, b) => (a.range.start <= b.range.start ? -1 : 1))

	// apply the replacements to the text
	let html = text
	let offset = 0
	for (let { range, scope } of replacements) {
		let replacementText = text.substring(range.start, range.end)
		let replacementHtml = `<span style="color: ${colors[scope]}">${replacementText}</span>`
		html =
			html.substring(0, offset + range.start) +
			replacementHtml +
			html.substring(offset + range.end)
		offset += replacementHtml.length - (range.end - range.start)
	}

	return html
}
