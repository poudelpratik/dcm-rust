'use strict';

import {Order, OrderedProduct, OrderManager} from "../CodeDistributor/exports.js";
import {uuidv4} from "../utils/helpers.js"
import {testData} from './test-data.js';

export default class OrderHistory {

    constructor() {
        OrderManager.new().then((orderManager) => {
            this.orderManager = orderManager;
            if (localStorage.getItem('orderHistory') !== null) {
                this.orderManager.orders = JSON.parse(localStorage.getItem('orderHistory'));
            }
            this.addFilterOptions();
            this.renderOrderHistory(this.orderManager.orders);
        });
        window.addEventListener('beforeunload', () => {
            localStorage.setItem('orderHistory', JSON.stringify(this.orderManager.orders));
        });
    }

    async update(products) {
        let orderedProducts = [];
        for (const product of products) {
            let orderedProduct = await OrderedProduct.new(uuidv4(), product.name, product.value, product.quantity);
            orderedProducts.push(orderedProduct);
        }
        let order = await Order.new(uuidv4(), orderedProducts);
        await this.orderManager.add(order);
        this.renderOrderHistory(this.orderManager.orders);
    }

    renderOrderHistory(orders) {
        const container = document.querySelector('#main-content2 div.orders-table');
        container.innerHTML = ''; // Clear previous content

        const table = document.createElement('table');
        table.id = 'orders-table';
        table.className = 'table table-striped';

        const thead = document.createElement('thead');
        thead.innerHTML = `
        <tr>
            <th>Order ID</th>
            <th>Products</th>
            <th>Total Price</th>
            <th>Archived</th>
            <th>Starred</th>
        </tr>`;
        table.appendChild(thead);

        const tbody = document.createElement('tbody');
        orders.forEach(order => {
            const tr = document.createElement('tr');

            tr.innerHTML = "";
            tr.innerHTML += `<td>${order.id}</td>`;
            tr.innerHTML += `<td>${order.products.map(p => `${p.name} (Qty: ${p.quantity})`).join(', ')}</td><td>${order.total.toFixed(2)}</td>`;

            const archiveTd = document.createElement('td');
            const archiveCheckbox = document.createElement('input');
            archiveCheckbox.type = 'checkbox';
            archiveCheckbox.checked = order.archived;
            archiveCheckbox.addEventListener('change', (event) => {
                event.preventDefault();
                this.handleArchiveChange(order.id).then(() => {
                    archiveCheckbox.checked = event.target.checked;
                })
            });
            archiveTd.appendChild(archiveCheckbox);
            tr.appendChild(archiveTd);

            const starredTd = document.createElement('td');
            const starredCheckbox = document.createElement('input');
            starredCheckbox.type = 'checkbox';
            starredCheckbox.checked = order.starred;
            starredCheckbox.addEventListener('change', (event) => {
                event.preventDefault();
                this.handleStarredChange(order.id).then(() => {
                    starredCheckbox.checked = event.target.checked;
                })
            });
            starredTd.appendChild(starredCheckbox);
            tr.appendChild(starredTd);

            tbody.appendChild(tr);
        });
        table.appendChild(tbody);

        container.appendChild(table);

        this.addExtraFeatures(container, orders);
    }

    async handleArchiveChange(orderId) {
        await this.orderManager.toggle_archive(orderId);
    }

    async handleStarredChange(orderId) {
        await this.orderManager.toggle_starred(orderId);
    }

    addFilterOptions() {
        const container = document.querySelector('#main-content2 div.filter');

        const filterOptionsDiv = document.createElement('div');
        filterOptionsDiv.id = 'filter-options';
        filterOptionsDiv.className = 'mb-3';
        filterOptionsDiv.innerHTML = `
        <div class="form-check form-check-inline">
            <input class="form-check-input" type="checkbox" id="filter-starred" value="starred">
            <label class="form-check-label" for="filter-starred">Starred</label>
        </div>
        <div class="form-check form-check-inline">
            <input class="form-check-input" type="checkbox" id="filter-archived" value="archived">
            <label class="form-check-label" for="filter-archived">Archived</label>
        </div>
        <button id="apply-filters" class="btn btn-primary">Apply Filters</button>
        <button id="reset-filters" class="btn btn-primary">Reset Filters</button>
        <button id="fill-test-data" class="btn btn-primary">Fill test data</button>
        <button id="delete-all-data" class="btn btn-primary">Delete all data</button>
        `;

        container.insertBefore(filterOptionsDiv, container.firstChild);

        document.getElementById('apply-filters').addEventListener('click', async () => {
            await this.applyFilters();
        });
        document.getElementById('fill-test-data').addEventListener('click', async () => {
            this.orderManager.orders.push(...testData);
            this.renderOrderHistory(this.orderManager.orders);
        });
        document.getElementById('delete-all-data').addEventListener('click', async () => {
            this.orderManager.orders = [];
            this.renderOrderHistory(this.orderManager.orders);
        });
        document.getElementById('reset-filters').addEventListener('click', async () => {
            document.getElementById('filter-starred').checked = false;
            document.getElementById('filter-archived').checked = false;
            this.renderOrderHistory(this.orderManager.orders);
        });
    }

    async applyFilters() {
        const isStarredChecked = document.getElementById('filter-starred').checked;
        const isArchivedChecked = document.getElementById('filter-archived').checked;
        let filterOptions = {};
        if (isStarredChecked) {
            filterOptions.starred = true;
        }
        if (isArchivedChecked) {
            filterOptions.archived = true;
        }
        const orders = await this.orderManager.filter(filterOptions);
        this.renderOrderHistory(orders);
    }

    addExtraFeatures(container, orders) {
        // Create main actions container
        const actionsDiv = document.createElement('div');
        actionsDiv.className = 'order-actions';
        actionsDiv.style.display = 'flex';
        actionsDiv.style.flexDirection = 'column';
        actionsDiv.style.alignItems = 'flex-start';
        actionsDiv.style.marginTop = '20px';

        // Function to create a button and result display
        const createFeature = (buttonText, onClick) => {
            const featureDiv = document.createElement('div');
            featureDiv.style.display = 'flex';
            featureDiv.style.alignItems = 'center';
            featureDiv.style.marginBottom = '10px';

            const button = document.createElement('button');
            button.textContent = buttonText;
            button.className = 'btn btn-primary';
            button.addEventListener('click', onClick);
            featureDiv.appendChild(button);

            const resultDisplay = document.createElement('span');
            resultDisplay.style.marginLeft = '15px';
            featureDiv.appendChild(resultDisplay);

            return {featureDiv, resultDisplay};
        };

        // Most invested Product feature
        const {
            featureDiv: mostSpentFeatureDiv,
            resultDisplay: mostSpentResultDisplay
        } = createFeature('See most invested product', async () => {
            let mostSpentProduct = await get_most_invested_product(orders); //call the imported function
            if (mostSpentProduct === null) {
                mostSpentResultDisplay.textContent = `No orders found`;
                return;
            }
            mostSpentResultDisplay.textContent = `${mostSpentProduct.name} (Total Amount: ${mostSpentProduct.total_amount})`;
        });

        // Most Bought Product feature
        const {
            featureDiv: mostBoughtFeatureDiv,
            resultDisplay: mostBoughtResultDisplay
        } = createFeature('See most bought product', async () => {
            let mostBoughtProduct = await get_most_bought_product(orders); //call the imported function
            if (mostBoughtProduct === null) {
                mostBoughtResultDisplay.textContent = `No orders found`;
                return;
            }
            mostBoughtResultDisplay.textContent = `${mostBoughtProduct.name} (Total Quantity: ${mostBoughtProduct.total_quantity})`;
        });

        // Append both features to the actions container
        actionsDiv.appendChild(mostSpentFeatureDiv);
        actionsDiv.appendChild(mostBoughtFeatureDiv);

        // Append the actions container to the main container
        container.appendChild(actionsDiv);
    }

}

function get_most_bought_product(orders) {
    const quantities = {};

    orders.forEach(order => {
        order.products.forEach(product => {
            quantities[product.name] = (quantities[product.name] || 0) + product.quantity;
        });
    });

    const mostBought = Object.entries(quantities).reduce((max, curr) => {
        return (curr[1] > (max[1] || 0)) ? curr : max;
    }, []);

    return mostBought.length ? {name: mostBought[0], total_quantity: mostBought[1]} : null;
}

function get_most_invested_product(orders) {
    const amounts = {};

    orders.forEach(order => {
        order.products.forEach(product => {
            amounts[product.name] = (amounts[product.name] || 0) + product.quantity * product.price;
        });
    });

    const mostInvested = Object.entries(amounts).reduce((max, curr) => {
        return (curr[1] > (max[1] || 0)) ? curr : max;
    }, []);

    return mostInvested.length ? {name: mostInvested[0], total_amount: mostInvested[1]} : null;
}
