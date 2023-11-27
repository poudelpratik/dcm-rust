import {initialize} from "./CodeDistributor/exports.js";

await initialize({
  codeDistributorDir: '/static/js/CodeDistributor/',
  codeDistributorPort: 51335,
  maxReconnectAttempts: 3,
});

export * from './CodeDistributor/exports.js';
