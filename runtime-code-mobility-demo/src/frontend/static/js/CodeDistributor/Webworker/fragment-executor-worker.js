self.importScripts('../vendor/message-pack/msgpack.js');

self.addEventListener('message', async (e) => {
    const { wasmBuffer, functionName, parameters } = e.data;
    try {
        const result = await execute(wasmBuffer, functionName, parameters);
        self.postMessage({ result });
    } catch (error) {
        self.postMessage({ error: error.message });
    }
});

function toLittleEndianBytes(value) {
    const buffer = new ArrayBuffer(4); // using 4 bytes for 32-bit integer
    const view = new DataView(buffer);
    view.setUint32(0, value, true); // the `true` parameter specifies little-endian
    return new Uint8Array(buffer);
}

async function execute(wasmModule, functionName, params) {
    let wrapperFunction = "execute__" + functionName;
    const instance = await WebAssembly.instantiate(wasmModule);
    let args = [];
    for (let param of params) {
        const param_as_bytes = self.msgpack.encode(param);
        const length = this.toLittleEndianBytes(param_as_bytes.length);
        while (args.length % 4 != 0) {
            args.push(0);
        }
        args.push(...length);
        args.push(...param_as_bytes);
    }

    // Allocate memory in the WebAssembly instance for the byte array.
    const ptr = instance.exports.alloc(args.length);

    // Get the memory of the WebAssembly instance as a Uint8Array.
    const memory = new Uint8Array(instance.exports.memory.buffer);
    memory.set(args, ptr);
    const resultPtr = instance.exports[wrapperFunction](ptr, params.length);

    // this.instance.exports.dealloc(ptr, bytes.length);

    const memory2 = new Uint8Array(instance.exports.memory.buffer);

    const length = new DataView(memory2.buffer, resultPtr, 4).getUint32(0, true);
    const resultData = new Uint8Array(memory2.buffer, resultPtr + 4, length);
    const result = await self.msgpack.decode(resultData);
    return result;
}
