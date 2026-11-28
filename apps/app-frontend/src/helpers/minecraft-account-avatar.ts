/**
 * Avatar URLs for a Minecraft account.
 *
 * `mc-heads.net` is unreachable on some networks, so the list starts with the
 * provider that is actually reachable and falls back from there.
 */
const LITTLESKIN_HOST = 'littleskin.cn'

export interface AccountAvatarInput {
	uuid: string
	name: string
	kind?: string
	apiRoot?: string | null
}

export function isLittleSkinAccount(input: Pick<AccountAvatarInput, 'kind' | 'apiRoot'>): boolean {
	const kind = String(input.kind ?? '')
	if (kind === 'littleskin' || kind === 'little_skin') return true
	return (
		kind === 'yggdrasil' &&
		String(input.apiRoot ?? '')
			.toLowerCase()
			.includes(LITTLESKIN_HOST)
	)
}

export function accountAvatarCandidates(input: AccountAvatarInput): string[] {
	const name = encodeURIComponent(input.name)
	const uuid = input.uuid
	const candidates: string[] = []

	if (isLittleSkinAccount(input)) {
		candidates.push(`https://littleskin.cn/avatar/player/${name}?size=64`)
	}

	candidates.push(
		`https://minotar.net/helm/${uuid}/64`,
		`https://mc-heads.net/avatar/${uuid}/64`,
		`https://crafatar.com/avatars/${uuid}?overlay&size=64`,
	)

	return candidates
}

export function accountAvatarUrl(input: AccountAvatarInput): string {
	return accountAvatarCandidates(input)[0]
}
