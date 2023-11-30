import {initialize} from "./CodeDistributor/exports.js";

let codeDistributorConfig = await fetch('/configuration');
codeDistributorConfig = await codeDistributorConfig.json();

await initialize({
    codeDistributorDir: '/static/js/CodeDistributor/',
    codeDistributorApiUrl: codeDistributorConfig.code_distributor_api_url,
    codeDistributorWsUrl: codeDistributorConfig.code_distributor_ws_url,
    maxReconnectAttempts: 3,
});

export * from './CodeDistributor/exports.js';
