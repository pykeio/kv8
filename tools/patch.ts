import { Command } from '@cliffy/command';

import { patch, updateSrc } from './_common.ts';

await new Command()
	.option('--no-reset', 'do not reset working tree')
	.action(async options => {
		await patch(!options.reset);
		await updateSrc();
	})
	.parse(Deno.args);
