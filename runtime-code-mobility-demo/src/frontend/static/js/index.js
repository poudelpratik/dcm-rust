'use strict';

//#region vendors global imports (everywhere available)
// https://currency.js.org/
import './vendor/currency/currency.js';
// https://github.com/nbubna/store#readme
import './vendor/store2/store2.min.js';
//#endregion
import {selectElement} from './utils/helpers.js';
import ProductCatalog from './ProductCatalog/index.js';
import MultistepForm from './MultistepForm.js';
import './Benchmarking.js';
import './initCodeDistributor.js';

async function main() {
    const productCatalog = new ProductCatalog();
    new MultistepForm(productCatalog);
    addNavigation();
}

function contentLoadedHandler(e) {
    window.removeEventListener('DOMContentLoaded', contentLoadedHandler, false);
    selectElement('body').classList.remove('preload');
    if (!window.Worker || !window.localStorage) {
        selectElement('body').innerHTML = `
  <p>Your browser requires WebWorker and LocalStorage functionalities in order to work.
  `;
        return;
    }
    console.log('DOM fully loaded and parsed');
    main.call(this);
}

if (document.readyState !== 'loading') {
    main.call(this);
} else {
    window.addEventListener('DOMContentLoaded', contentLoadedHandler, false);
}


function addNavigation() {
    document.getElementById('linkWebShop').addEventListener('click', function () {
        document.getElementById('main-content').hidden = false;
        document.getElementById('main-content3').hidden = true;
        document.getElementById('main-content2').hidden = true;
        // this.classList.add('active');
        // document.getElementById('linkOrderHistory').classList.remove('active');
        document.getElementById('page-title').innerText = 'Web Shop';
    });

    document.getElementById('linkOrderHistory').addEventListener('click', function () {
        document.getElementById('main-content').hidden = true;
        document.getElementById('main-content3').hidden = true;
        document.getElementById('main-content2').hidden = false;
        // this.classList.add('active');
        // document.getElementById('linkWebShop').classList.remove('active');
        document.getElementById('page-title').innerText = 'Order History';
    });

    document.getElementById('linkPlayground').addEventListener('click', function () {
        document.getElementById('main-content').hidden = true;
        document.getElementById('main-content2').hidden = true;
        document.getElementById('main-content3').hidden = false;
        // this.classList.add('active');
        // document.getElementById('linkWebShop').classList.remove('active');
        document.getElementById('page-title').innerText = 'WASM Playground';
    });
}