// The standard suite, reusing the last e2e build when it is still current:
// `npm run test:e2e:reuse` (add `-- --spec e2e/specs/<file>` for one spec).
// The same as `ARM_E2E_REUSE_BUILD=1 npm run test:e2e`, without an environment
// prefix, so it works from any shell. A local fast-rerun aid: the gate never
// uses it. See `build-freshness.js` for when a build counts as current.

import { config as standardConfig, prepareRun } from './wdio.conf.js';

export const config = {
  ...standardConfig,
  onPrepare: async () => prepareRun({ reuse: true }),
};
