import {factorial, fibonacci} from "./CodeDistributor/exports.js";

function factorial_js(n) {
    if (n === 0) return 1;
    return n * factorial_js(n - 1);
}

function fibonacci_js(n) {
    if (n === 0) return 0;
    if (n === 1) return 1;
    return fibonacci_js(n - 1) + fibonacci_js(n - 2);
}

async function benchmarkFunction(func, param, times) {
    let durations = [];

    for (let i = 0; i < times; i++) {
        const startTime = performance.now();

        // We assume that the function returns a promise
        await func(param);
        const endTime = performance.now();

        durations.push(endTime - startTime);
    }

    // Calculate mean, median, and standard deviation
    const mean = durations.reduce((a, b) => a + b, 0) / durations.length;
    const sortedDurations = durations.slice().sort((a, b) => a - b);
    const median = sortedDurations[Math.floor(durations.length / 2)];
    const stdDev = Math.sqrt(durations.map(x => Math.pow(x - mean, 2)).reduce((a, b) => a + b) / durations.length);
    const min = sortedDurations[0];
    const max = sortedDurations[sortedDurations.length - 1];

    return {
        mean: mean.toFixed(2),
        median: median.toFixed(2),
        stdDev: stdDev.toFixed(2),
        min: min.toFixed(2),
        max: max.toFixed(2)
    };
}

async function benchmarkAll(functionName, param, times) {
    // Benchmark the WebAssembly version
    const wasmResults = await benchmarkFunction(window.benchmarks[functionName], param, times);
    console.log(`${functionName} WebAssembly Results: `, wasmResults);

    // Benchmark the JavaScript version
    const jsFunctionName = functionName + '_js';
    const jsResults = await benchmarkFunction(window.benchmarks[jsFunctionName], param, times);
    console.log(`${functionName} JavaScript Results: `, jsResults);
}

window.benchmarks = {
    benchmarkAll,
    factorial,
    fibonacci,
    fibonacci_js,
    factorial_js
};