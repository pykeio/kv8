import { Command } from '@cliffy/command';

import { patch } from './_common.ts';

await new Command()
	.option('--no-reset', 'do not reset working tree')
	.action(async options => {
		await patch(!options.reset);
	})
	.parse(Deno.args);
