import { ref } from 'vue'

/**
 * Welcome text template shown above the instance library.
 *
 * Mini markup:
 * - `\` toggles between text and code. Everything between two `\` is code.
 * - A code block made of `#`…`######` turns the following text into a heading
 *   level 1…6. Same for `@`…, which renders smaller subtitle text.
 * - `$name` is replaced with a value: `minecraft_name`, `launcher_version`,
 *   `last_play_name`, `last_play_version`, `last_play_time`, `game_time`,
 *   `last_world`, `time` and `date`. `${name}` is accepted as well.
 * - Newlines and spaces are kept as typed.
 */
export const DEFAULT_WELCOME_TEMPLATE = [
	'\\#\\欢迎 $minecraft_name',
	'\\@@@@\\通过 Alemona Launcher 启动并管理 Minecraft，下载内容并通过 Minecraft Explore 探索更多功能！',
].join('\n')

const STORAGE_KEY = 'alemona.home-welcome-template'

function readStoredTemplate(): string {
	try {
		return localStorage.getItem(STORAGE_KEY) ?? DEFAULT_WELCOME_TEMPLATE
	} catch {
		return DEFAULT_WELCOME_TEMPLATE
	}
}

export const welcomeTemplate = ref<string>(readStoredTemplate())

export function setWelcomeTemplate(value: string): void {
	welcomeTemplate.value = value
	try {
		localStorage.setItem(STORAGE_KEY, value)
	} catch {
		// Keep the value in memory when storage is unavailable.
	}
}

export function resetWelcomeTemplate(): void {
	setWelcomeTemplate(DEFAULT_WELCOME_TEMPLATE)
}

export type WelcomeVars = Record<string, string | undefined>

export type WelcomeBlock =
	| { kind: 'heading'; level: number; text: string }
	| { kind: 'subtitle'; level: number; text: string }
	| { kind: 'text'; text: string }

const HEADING_MARKER = /^(#{1,6})([\s\S]*)$/
const SUBTITLE_MARKER = /^(@{1,6})([\s\S]*)$/

function trimBlock(text: string): string {
	return text.replace(/^[\r\n]+|[\r\n]+$/g, '')
}

/**
 * Splits a template into renderable blocks. A code block that only contains a
 * marker (`\#\` or `\@@@@\`) styles the text that follows it, while a marker in
 * front of text in the same code block (`\#Heading\`) styles that text directly.
 */
export function welcomeBlocks(template: string): WelcomeBlock[] {
	const blocks: WelcomeBlock[] = []
	let style: { kind: 'heading' | 'subtitle'; level: number } | null = null
	let text = ''
	let code = ''
	let inCode = false

	function flush(): void {
		const trimmed = trimBlock(text)
		text = ''
		if (!trimmed.trim()) return
		blocks.push(
			style
				? { kind: style.kind, level: style.level, text: trimmed }
				: { kind: 'text', text: trimmed },
		)
	}

	function applyCode(raw: string): void {
		const heading = HEADING_MARKER.exec(raw)
		if (heading) {
			flush()
			style = { kind: 'heading', level: heading[1].length }
			text = heading[2] ?? ''
			return
		}
		const subtitle = SUBTITLE_MARKER.exec(raw)
		if (subtitle) {
			flush()
			style = { kind: 'subtitle', level: subtitle[1].length }
			text = subtitle[2] ?? ''
			return
		}
		text += raw
	}

	for (let index = 0; index < template.length; index += 1) {
		const char = template[index]
		if (char === '\\') {
			const next = template[index + 1]
			if (next === undefined || next === '\n' || next === '\r') {
				// A stray escape at the end of a line is ignored.
				continue
			}
			if (inCode) {
				inCode = false
				applyCode(code)
				code = ''
			} else {
				inCode = true
			}
			continue
		}
		if (inCode) {
			code += char
		} else {
			text += char
		}
	}
	if (inCode && code) applyCode(code)
	flush()

	return blocks
}

export function substituteWelcomeVars(text: string, vars: WelcomeVars): string {
	return text.replace(
		/\$(\{([a-z_]+)\}|([a-z_]+))/gi,
		(match: string, _wrapped: string, braced: string, plain: string) => {
			const name = (braced || plain || '').toLowerCase()
			const value = vars[name]
			return value === undefined ? match : value
		},
	)
}

/** Formats a playtime in seconds as `12h 34m`. */
export function formatPlaytime(seconds: number | null | undefined): string {
	if (!seconds || seconds <= 0) return ''
	const hours = Math.floor(seconds / 3600)
	const minutes = Math.floor((seconds % 3600) / 60)
	if (hours <= 0) return `${minutes}m`
	return `${hours}h ${minutes}m`
}
