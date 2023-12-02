const DEBUG = false;
const API_KEY = 'tZwqxgVXSEaqYQZ';
let ALERT_TIMEOUT = null;
let browser = chrome;

const isUndefined = (variable) =>
    typeof variable === 'undefined' || variable === null;

/**
 * @param {String} htmlString representing a single element
 * @return {Element}
 */
const convertHtmlStringToElement = (htmlString) => {
    var template = document.createElement('template');
    htmlString = htmlString.trim(); // Never return a text node of whitespace as the result

    template.innerHTML = htmlString;
    return template.content.firstChild;
};

const loadDataFromLastSession = () => {
    browser.storage.local.get('ClientID', function (items) {
        if (!browser.runtime.lastError) {
            const clientId = items.ClientID;
            if (!isUndefined(clientId)) {
                document.querySelector('#input_clientid').value = clientId;
            }
        }
    });

    browser.storage.local.get('CodeDistributorHost', function (items) {
        if (!browser.runtime.lastError) {
            const codeDistributorHost = items.CodeDistributorHost;
            if (!isUndefined(codeDistributorHost)) {
                document.querySelector('#input_code_distributor_host').value = codeDistributorHost;
            }
        }
    });

    browser.storage.local.get('Client', function (items) {
        if (!browser.runtime.lastError) {
            const client = items.Client;
            if (!isUndefined(client) && !client['error']) {
                document.querySelector('#fragment-table').classList.remove('d-none');
                fillFragmentTable(client);
            }
        }
    });
};

const fillFragmentTable = (client) => {
    const fragmentTable = document.querySelector('#fragment-table');
    const fragmentTableBody = fragmentTable.querySelector('tbody');
    let rows = [];

    const createSerialNumberCell = (value) => {
        const cell = document.createElement('td');
        cell.classList.add('col-2');
        cell.textContent = value;
        return cell;
    };
    const createFragmentIdCell = (value) => {
        const cell = document.createElement('td');
        cell.classList.add('col-4');
        cell.textContent = value;
        return cell;
    };
    const createFragmentStatus = (value) => {
        const cell = document.createElement('td');
        cell.classList.add('col-4');
        let location = ''; // default = server

        if (value === 'Client') {
            location = 'checked';
        }

        const toggleElementString = `
    <label class="toggleSwitch nolabel">
      <input type="checkbox" ${location} />
      <a></a>
      <span>
        <span class="left-span">Server</span>
        <span class="right-span">Client</span>
      </span>
    </label>`;

        const toggleElement = convertHtmlStringToElement(toggleElementString);
        cell.append(toggleElement);
        return cell;
    };

    let serial = 1;
    client.fragments.forEach((fragment) => {
        const row = document.createElement('tr');
        row.classList.add('d-flex');
        const serialNumberCell = createSerialNumberCell(serial++);
        const fragmentIDCell = createFragmentIdCell(fragment.id);
        const fragmentStatusCell = createFragmentStatus(fragment.execution_location);
        row.append(serialNumberCell, fragmentIDCell, fragmentStatusCell);
        rows.push(row);
    });

    fragmentTableBody.innerHTML = '';
    fragmentTableBody.append(...rows);
};

/**
 *  Like window.location it returns the current URL.
 * @return {Promise<URL>}
 */
const getCurrentUrl = () => {
    return new Promise((resolve, reject) => {
        browser.tabs.query(
            {active: true, currentWindow: true, lastFocusedWindow: !DEBUG},
            (tabs) => {
                if (tabs.length === 0) {
                    reject(new Error('no active tab found'));
                    return;
                }

                let {url: urlString} = tabs[0];

                if (!urlString) {
                    reject(new Error('failed to retrieve URL'));
                    return;
                }

                resolve(new URL(urlString));
            },
        );
    });
};

const createSuccessAlert = (msg) => {
    const div = document.createElement('div');
    div.classList.add('my-alert', 'success');
    const svg = convertHtmlStringToElement(`
  <svg
    class="bi flex-shrink-0 me-2"
    width="16"
    height="16"
    role="img"
    aria-hidden="true"
  >
    <use xlink:href="#check-circle-fill" />
  </svg>
  `);

    const alertPrefix = convertHtmlStringToElement(
        ` <strong class="me-2"> Success </strong>`,
    );

    div.append(svg, alertPrefix, msg);
    return div;
};

const createErrorAlert = (msg) => {
    const div = document.createElement('div');
    div.classList.add('my-alert', 'error');
    const svg = convertHtmlStringToElement(`
  <svg
    class="bi flex-shrink-0 me-2"
    width="16"
    height="16"
    role="img"
    aria-hidden="true"
  >
    <use xlink:href="#exclamation-triangle-fill" />
  </svg>
  `);

    const alertPrefix = convertHtmlStringToElement(
        ` <strong class="me-2"> Error </strong>`,
    );

    div.append(svg, alertPrefix, msg);
    return div;
};

const hideAlert = () => {
    const alert = document.querySelector('#alert');
    alert.classList.remove('show');
    alert.innerHTML = '';
    // reset animation
    alert.getAnimations();
};

// type: success || error
const showAlert = (type, msg = '') => {
    const alert = document.querySelector('#alert');
    hideAlert();

    // alert.classList.remove('show'); // for current existing showing errors
    let alertElement = undefined;

    if (type === 'success') alertElement = createSuccessAlert(msg);
    else if (type === 'error') alertElement = createErrorAlert(msg);
    else return;

    alert.append(alertElement);
    alert.classList.add('show');

    // remove class and clear children after 5s
    if (ALERT_TIMEOUT) {
        clearTimeout(ALERT_TIMEOUT);
    }
    ALERT_TIMEOUT = setTimeout(hideAlert, 5000);
};

const updateClientHandler = async (evt) => {
    const clientId = document.querySelector('#input_clientid').value;
    const codeDistributorHost = document.querySelector('#input_code_distributor_host').value;
    const fragmentTableBody = document.querySelector('#fragment-table tbody');
    const rows = Array.from(fragmentTableBody.children);
    let fragments = [];

    // building the fragments array
    rows.forEach((row) => {
        const cols = row.children;
        let fragment = {};
        fragment['id'] = cols[1].textContent;
        fragment['execution_location'] = cols[2].querySelector('input').checked
            ? 'Client'
            : 'Server';
        fragments.push(fragment);
    });

    if (fragments.length === 0) {
        showAlert('error', 'missing data to update');
        return;
    }

    let url = '';
    try {
        url = await getCurrentUrl();
    } catch (err) {
        showAlert('error', err);
        console.error(err);
        return;
    }

    try {
        let url = new URL(codeDistributorHost);
        url.pathname = `api/clients/${clientId}`;

        const response = await fetch(url, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                Origin: url.protocol + url.host,
                'X-Api-Key': API_KEY,
            },
            body: JSON.stringify(fragments),
        });
        await response.json();
        showAlert('success', 'Fragments updated.');
    } catch (err) {
        showAlert('error', 'update request failed');
        console.error(err);
    }
};

const loadClientHandler = async (evt) => {
    const clientId = document.querySelector('#input_clientid').value;
    const codeDistributorHost = document.querySelector('#input_code_distributor_host').value;
    if (isUndefined(clientId) || clientId === '') {
        return;
    }

    browser.storage.local.set({ClientID: clientId}, function () {
        if (browser.runtime.lastError) {
            console.log("Error: couldn't save client id.");
        }
    });

    let url = '';
    try {
        url = await getCurrentUrl();
    } catch (err) {
        showAlert('error', err);
        console.error(err);
        return;
    }

    let client = undefined;

    try {
        let url = new URL(codeDistributorHost);
        url.pathname = `api/clients/${clientId}`;
        const response = await fetch(url, {
            method: 'GET',
            headers: {
                'X-Api-Key': API_KEY,
            },
        });

        client = await response.json();
        showAlert('success', 'Client loaded.');
    } catch (err) {
        showAlert('error', 'fetch request failed');
        browser.storage.local.remove('Client', function () {
            if (browser.runtime.lastError) {
                console.error(browser.runtime.lastError);
            }
        });
        console.error(err);
        return;
    }

    if (!client['error']) {
        browser.storage.local.set({Client: client}, function () {
            if (browser.runtime.lastError) {
                console.log("Error: couldn't save client id.");
            }
        });
        document.querySelector('#fragment-table').classList.remove('d-none');
        fillFragmentTable(client);
    } else {
        const fragmentTable = document.querySelector('#fragment-table');
        fragmentTable.classList.add('d-none');
        const fragmentTableBody = fragmentTable.querySelector('tbody');
        fragmentTableBody.innerHTML = '';
        showAlert('error', client['error'].message);
        browser.storage.local.remove('Client', function () {
            if (browser.runtime.lastError) {
                console.error(browser.runtime.lastError);
            }
        });
        browser.storage.local.remove('ClientID', function () {
            if (browser.runtime.lastError) {
                console.error(browser.runtime.lastError);
            }
        });
    }
};

const fetchClientsHandler = async (evt) => {
    let url = '';
    const codeDistributorHost = document.querySelector('#input_code_distributor_host').value;
    try {
        url = await getCurrentUrl();
    } catch (err) {
        showAlert('error', err.message);
        console.error(err);
        return;
    }

    let clients = [];

    try {
        let url = new URL(codeDistributorHost);
        url.pathname = `api/clients`;

        const response = await fetch(url, {
            method: 'GET',
            headers: {
                'X-Api-Key': API_KEY,
            },
        });

        clients = await response.json();
    } catch (err) {
        showAlert('error', 'fetch request failed');
        console.error(err);
        return;
    }

    const clientsTable = document.querySelector('#clients-table');
    const clientsTableBody = clientsTable.querySelector('tbody');

    if (clients?.length !== 0) {
        clientsTable.classList.remove('d-none');
    } else {
        clientsTableBody.innerHTML = '';
        clientsTable.classList.add('d-none');
        return;
    }

    let rows = [];

    const createClientPosCell = (value) => {
        const cell = document.createElement('th');
        cell.classList.add('text-center', 'col-1');
        cell.setAttribute('scope', 'row');
        cell.textContent = value;
        return cell;
    };
    const createClientUUIDCell = (value) => {
        const cell = document.createElement('td');
        cell.classList.add('col');
        cell.textContent = value;
        return cell;
    };

    let serial = 1;
    clients.forEach((fragment) => {
        const row = document.createElement('tr');
        row.classList.add('d-flex');
        const nr = createClientPosCell(serial++);
        const uuid = createClientUUIDCell(fragment['uuid']);
        row.append(nr, uuid);
        rows.push(row);
    });

    clientsTableBody.innerHTML = '';
    clientsTableBody.append(...rows);
};

function main() {
    loadDataFromLastSession();

    let forms = document.querySelectorAll('.form');
    [...forms].forEach((form) => {
        form.addEventListener('submit', (evt) => {
            evt.preventDefault(); // stop reloading page

        });
    });

    let input_code_distributor_host = document.querySelector('#input_code_distributor_host');
    input_code_distributor_host.addEventListener('input', () => {
        browser.storage.local.set({CodeDistributorHost: input_code_distributor_host.value}, function () {
            if (browser.runtime.lastError) {
                console.log("Error: couldn't save code distributor host.");
            }
        });
    }, false);

    let fetchClientsButton = document.querySelector('#fetchClientsButton');
    fetchClientsButton.addEventListener('click', fetchClientsHandler, false);

    let loadClientButton = document.querySelector('#loadClientButton');
    loadClientButton.addEventListener('click', loadClientHandler, false);

    let updateClientButton = document.querySelector('#updateClientButton');
    updateClientButton.addEventListener('click', updateClientHandler, false);

    document
        .querySelector('input[type=search]')
        .addEventListener('input', (evt) => {
            // clear button in search input field
            // Object.getPrototypeOf(evt).constructor.name
            if (!(evt instanceof InputEvent)) {
                browser.storage.local.remove(['Client'], function () {
                    if (browser.runtime.lastError) {
                        console.error(browser.runtime.lastError);
                    }
                });
                const fragmentTable = document.querySelector('#fragment-table');
                fragmentTable.classList.add('d-none');
                const fragmentTableBody = fragmentTable.querySelector('tbody');
                fragmentTableBody.innerHTML = '';
            }
        });
}

function contentLoadedHandler(e) {
    window.removeEventListener('DOMContentLoaded', contentLoadedHandler, false);

    main.call(this);
}

window.addEventListener('DOMContentLoaded', contentLoadedHandler, false);
