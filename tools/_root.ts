import { fromFileUrl, dirname } from '@std/path';

const root = dirname(dirname(fromFileUrl(import.meta.url)));
export default root;
