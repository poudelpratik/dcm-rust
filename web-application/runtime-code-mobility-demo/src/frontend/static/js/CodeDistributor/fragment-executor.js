import './vendor/message-pack/msgpack.js';
import WebworkerPool from "./Webworker/WebworkerPool.js";

export default class FragmentExecutor {
    fragmentMap = new Map();

    constructor(fragmentRegistry, configuration) {
        this.configuration = configuration;
        this.fragmentRegistry = fragmentRegistry;
        const maxPoolSize = navigator.hardwareConcurrency || 1;
        this.webworkerPool = new WebworkerPool(maxPoolSize, `${this.configuration.codeDistributorDir}Webworker/fragment-executor-worker.js`);
    }

    async init() {
        await this.loadFragments();
    }

    async loadFragments() {
        for (const id of this.fragmentRegistry.fragmentMap.keys()) {
            await this.loadFragment(id);
        }
    }

    toLittleEndianBytes(value) {
        const buffer = new ArrayBuffer(4); // using 4 bytes for 32-bit integer
        const view = new DataView(buffer);
        view.setUint32(0, value, true); // the `true` parameter specifies little-endian
        return new Uint8Array(buffer);
    }

    async loadFragment(id) {
        let identifier = `${id}.wasm`;
        const response = await fetch(`${this.configuration.codeDistributorDir}fragments/${identifier}`);
        const moduleBytes = await response.arrayBuffer();
        const compiledModule = await WebAssembly.compile(moduleBytes);
        this.fragmentMap.set(id, compiledModule);
    }

    async execute(fragmentId, functionName, parameters) {
        return new Promise((resolve, reject) => {
            const worker = this.webworkerPool.getWorker();
            const wasmBuffer = this.fragmentMap.get(fragmentId);

            if (!wasmBuffer) {
                return reject(new Error(`WASM module for fragment ${fragmentId} not found`));
            }

            worker.onmessage = (e) => {
                if (e.data.error) {
                    reject(new Error(e.data.error));
                } else {
                    const result = e.data.result;
                    resolve(result);
                }
                this.webworkerPool.releaseWorker(worker);
            };

            worker.onerror = (err) => {
                reject(err);
                this.webworkerPool.releaseWorker(worker);
            };

            worker.postMessage({
                wasmBuffer: wasmBuffer,
                functionName: functionName,
                parameters: parameters,
            });
        });
    }
}
