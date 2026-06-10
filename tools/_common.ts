import $ from '@david/dax';
import { copy } from '@std/fs';
import { dirname, fromFileUrl, join } from '@std/path';

export const ROOT = dirname(dirname(fromFileUrl(import.meta.url)));

export const PATCHES_DIR = join(ROOT, 'patches');
export const SRC_DIR = join(ROOT, 'src');
export const TARGET_DIR = join(ROOT, 'rusty_v8');
export const TARGET_SRC_DIR = join(TARGET_DIR, 'src');

export async function resetUpstream() {
	await $`git reset --hard HEAD`
		.cwd(TARGET_DIR);
	await $`git submodule foreach --recursive git reset --hard HEAD`
		.cwd(TARGET_DIR);
	await $`git clean -fdx`
		.cwd(TARGET_DIR);
}

export async function patch(noReset: boolean = false) {
	if (!noReset) {
		await resetUpstream();
	}

	for await (const file of Deno.readDir(PATCHES_DIR)) {
		if (!file.isFile || !file.name.endsWith('.patch')) {
			continue;
		}

		await $`git apply ${join(PATCHES_DIR, file.name)} --ignore-whitespace --recount --verbose`.cwd(TARGET_DIR);
	}
}

export async function updateSrc() {
	await copy(TARGET_SRC_DIR, SRC_DIR, { overwrite: true });

	const icuPath = 'third_party/icu/common/icudtl.dat';
	await Deno.mkdir(join(ROOT, dirname(icuPath)), { recursive: true });
	await copy(join(TARGET_DIR, icuPath), join(ROOT, icuPath), { overwrite: true });
}

export async function latestCommit(): Promise<[commit: string, version: string]> {
	const res = (await $`git ls-remote --refs --sort='-version:refname' origin`.cwd(TARGET_DIR).stdout('piped'));
	const latest = res.stdout.split('\n')[0];
	const [ commit, ref ] = latest.split('\t');
	return [ commit, ref.replace(/^refs\/tags\/v?/, '').replace(/\^\{}$/, '') ];
}
