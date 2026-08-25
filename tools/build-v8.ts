import $ from '@david/dax';
import { Command } from '@cliffy/command';

import { TARGET_DIR } from './_common.ts';

const FEATURES = ['v8_enable_pointer_compression'];

await new Command()
	.option('--target <target:string>', 'rustc target', { required: true })
	.action(async options => {
		await $`cargo build --release --features ${FEATURES.join(',')} --target ${options.target} -vvv`
			.cwd(TARGET_DIR)
			.env({ V8_FROM_SOURCE: 'true' });
	})
	.parse(Deno.args);
