import {count_vowels, factorial, fibonacci, find_primes} from "./initCodeDistributor.js";

function execute_factorial() {
    let n = parseInt(document.getElementById("factorial_input").value);
    factorial(n).then(result => {
        document.getElementById("factorial_result").innerHTML = result;
    });
}

function execute_count_vowels() {
    let s = document.getElementById("count_vowel_input").value;
    count_vowels(s).then(result => {
        document.getElementById("count_vowel_result").innerHTML = result;
    });
}

function execute_find_primes() {
    let n = parseInt(document.getElementById("find_primes_input").value);
    find_primes(n).then(result => {
        document.getElementById("find_primes_result").innerHTML = result;
    });
}

function execute_fibonacci() {
    let n = parseInt(document.getElementById("fibonacci_input").value);
    fibonacci(n).then(result => {
        document.getElementById("fibonacci_result").innerHTML = result;
    });
}

document.getElementById("factorial_calc").addEventListener("click", execute_factorial);
document.getElementById("fibonacci_calc").addEventListener("click", execute_fibonacci);
document.getElementById("find_primes_calc").addEventListener("click", execute_find_primes);
document.getElementById("count_vowel_calc").addEventListener("click", execute_count_vowels);

