import {EXECUTION_LOCATION} from "./constants.js";

export default class DecisionSystemSimulator {
    popupBox = null; // This will hold the popup box element

    constructor(configuration, auth, fragmentRegistry) {
        this.fragmentRegistry = fragmentRegistry;
        this.configuration = configuration;
        this.auth = auth;
        this.createPopupUI(); // Call the function to create the UI when the class is instantiated
        this.populateFunctionSelect(); // Populate the select dropdown
        document.querySelector('#decisionSystemSimulator').addEventListener('click', () => this.showPopup());
    }

    // Method to create the popup UI and append it to the document body
    createPopupUI() {
        const overlay = document.createElement('div');
        overlay.id = 'overlay';
        overlay.className = 'overlay';
        overlay.addEventListener('click', () => this.closePopup());

        this.popupBox = document.createElement('div');
        this.popupBox.id = 'popupBox';
        this.popupBox.className = 'popup-box container';
        this.popupBox.innerHTML = `
        <h1>Select Execution Location</h1>
        <div class="form-check form-switch">
            <input class="form-check-input" type="checkbox" role="switch" id="executionLocation">
            <label class="form-check-label" for="executionLocation">Client/Server</label>
        </div>
        <div class="fragmentSelector">
        <label for="fragmentSelect">Select a Fragment:</label>
        <select id="fragmentSelect"></select>
        <br/>
        <button class="btn btn-info" id="closePopup">Close</button>
        </div>
    `;

        document.body.appendChild(overlay);
        document.body.appendChild(this.popupBox);

        document.getElementById('closePopup').addEventListener('click', () => this.closePopup());

        const style = document.createElement('style');
        style.textContent = `
    .site-header {
        position: fixed;
        max-width: 100%;
        margin: 0 auto;
        padding: 20px;
        box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
        background-color: lightblue;
        border-radius: 10px;
    }
    h1 {
        text-align: center;
    }
    #fragmentSelect {
        display: block;
        margin: 10px 0 5px;
        width: 30%;
        padding: 10px;
        margin-bottom: 10px;
        border-radius: 5px;
        border: 1px solid #ddd;
    }
    button:hover {
        background-color: #45a049;
    }
    .popup-box {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        z-index: 1000;
        display: none;
        background-color: #fff;
    }
    `;
        document.head.appendChild(style);
        document.getElementById('executionLocation').addEventListener('change', () => this.executionLocationUpdated());
        document.getElementById('fragmentSelect').addEventListener('change', () => this.fragmentSelected());
    }


    // Method to show the popup box and the overlay
    showPopup() {
        this.popupBox.style.display = 'block';
        document.getElementById('overlay').style.display = 'block';
    }

    // Method to close the popup box and the overlay
    closePopup() {
        this.popupBox.style.display = 'none';
        document.getElementById('overlay').style.display = 'none';
    }

    // Method to update the execution location based on the checkbox
    async executionLocationUpdated() {
        const executionLocation = document.getElementById('executionLocation').checked ? 'Server' : 'Client';
        const selectedFuncId = document.getElementById('fragmentSelect').value;
        await fetch(this.configuration.codeDistributorApiUrl + 'client/' + this.auth.client_id, {
            headers: {
                'Content-Type': 'application/json',
                'X-Api-Key': this.configuration.codeDistributorApiKey,
            },
            method: 'PUT',
            body: JSON.stringify([{id: selectedFuncId, execution_location: executionLocation}])
        });
    }

    // Method to populate the function selection dropdown
    populateFunctionSelect() {
        const fragmentSelect = document.getElementById('fragmentSelect');
        // Clear existing options
        fragmentSelect.innerHTML = '';
        this.fragmentRegistry.fragmentMap.forEach((value, key) => {
            const option = document.createElement('option');
            option.value = key;
            option.textContent = key;
            fragmentSelect.appendChild(option);
        });
    }

    fragmentSelected() {
        const functionSelect = document.getElementById('fragmentSelect');
        const selectedFragmentId = functionSelect.value;
        const executionLocation = this.fragmentRegistry.fragmentMap.get(selectedFragmentId);
        if (executionLocation == EXECUTION_LOCATION.SERVER) {
            document.getElementById('executionLocation').checked = true;
        } else {
            document.getElementById('executionLocation').checked = false;
        }
    }
}
