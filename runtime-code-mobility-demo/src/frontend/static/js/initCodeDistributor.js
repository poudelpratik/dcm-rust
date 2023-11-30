import {initialize} from "./CodeDistributor/exports.js";

await initialize({
  codeDistributorDir: '/static/js/CodeDistributor/',
  codeDistributorApiUrl: 'http://localhost:51335/api/',
  codeDistributorWsUrl: 'ws://localhost:51335/ws',
  maxReconnectAttempts: 3,
});

export * from './CodeDistributor/exports.js';
