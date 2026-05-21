import $ from '@david/dax';
import { join } from '@std/path';

import { latestCommit, resetUpstream, ROOT, TARGET_DIR, updateSrc, patch } from './_common.ts';

const CARGO_TOML = join(ROOT, 'Cargo.toml');
const manifest = await Deno.readTextFile(CARGO_TOML);
const versionLine = manifest.match(/version = "(\d+\.\d+\.\d+)"/)!;
const currentVersion = versionLine[1];

await $`git fetch`.cwd(TARGET_DIR);

const [ newCommit, latestVersion ] = await latestCommit();
if (latestVersion === currentVersion) {
	console.log('Up to date.');
	Deno.exit(0);
}

console.log(`Updating to new version: ${latestVersion} (${newCommit}) from ${currentVersion}`);

await resetUpstream();

await $`git checkout ${newCommit}`.cwd(TARGET_DIR);
await $`git submodule update --init --recursive`.cwd(TARGET_DIR);

await patch(true);

await updateSrc();

await Deno.writeTextFile(
	CARGO_TOML,
	manifest.replace(versionLine[0], `version = "${latestVersion}"`)
);

await $`git add rusty_v8 src Cargo.toml`.cwd(ROOT);
await $`git commit -m ${`Update to v8 ${latestVersion}`}`.cwd(ROOT);
await $`git push origin +HEAD:refs/heads/autoupdate`.cwd(ROOT);
await $`git fetch origin autoupdate`.cwd(ROOT);

const res = await $`gh pr view autoupdate --json state`.stdout('piped').noThrow();
if (res.code === 0 && res.stdoutJson.state === 'OPEN') {
	if (!res.stdoutJson.isDraft) {
		await $`gh pr ready autoupdate --undo`.noThrow();
	}
	await $`gh pr edit autoupdate --title ${`Update to v8 ${latestVersion}`}`.cwd(ROOT);
} else {
	await $`gh pr create --draft --title ${`Update to v8 ${latestVersion}`} --body "" --head pykeio:autoupdate`.cwd(ROOT);
}
