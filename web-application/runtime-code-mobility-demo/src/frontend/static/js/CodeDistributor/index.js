import WebSocketClient, {MESSAGE_TYPES} from "./websocketclient.js";
import FragmentRegistry from "./fragment-registry.js";
import FragmentExecutor from "./fragment-executor.js";
import DecisionSystemSimulator from "./decision-system-simulator.js";
import {EXECUTION_LOCATION} from "./constants.js";

export default class CodeDistributionManager {

    constructor(configuration) {
        this.configuration = configuration;
        this.webSocketClient = new WebSocketClient(configuration);
        this.fragmentRegistry = new FragmentRegistry(configuration);
    }

    async init() {
        await this.fragmentRegistry.init();
        let storedAuth = localStorage.getItem('auth');
        let auth;
        if (storedAuth === null) {
            auth = await fetch(this.configuration.codeDistributorApiUrl + 'auth', {
                method: 'POST',
                headers: {
                    'X-Api-Key': this.configuration.codeDistributorApiKey,
                }
            });
            auth = await auth.json();
            localStorage.setItem('auth', JSON.stringify(auth));
        } else {
            storedAuth = JSON.parse(storedAuth);
            auth = await fetch(this.configuration.codeDistributorApiUrl + 'auth', {
                method: 'POST',
                headers: {
                    'X-Authorization': 'Bearer ' + storedAuth.token,
                    'X-Api-Key': this.configuration.codeDistributorApiKey,
                }
            });
            auth = await auth.json();
            localStorage.setItem('auth', JSON.stringify(auth));
        }

        await this.webSocketClient.init(auth);
        this.fragmentExecutor = new FragmentExecutor(this.fragmentRegistry, this.configuration);
        await this.fragmentExecutor.init();
        new DecisionSystemSimulator(this.configuration, auth, this.fragmentRegistry);
    }

    async execute(fragmentId, functionName, parameters) {
        const clientIcon = document.querySelector('#client-icon');
        const serverIcon = document.querySelector('#server-icon');

        let fragmentExecutionLocation = this.fragmentRegistry.fragmentMap.get(fragmentId);
        let data = {
            fragment_id: fragmentId,
            function_name: functionName,
            parameters: parameters
        };
        if (fragmentExecutionLocation == EXECUTION_LOCATION.SERVER && this.webSocketClient.isOpen()) {
            serverIcon.classList.add('trigger');
            setTimeout(() => {
                serverIcon.classList.remove('trigger');
            }, 200);
            let result = await this.webSocketClient.sendMessage(MESSAGE_TYPES.EXECUTE_FUNCTION, data);
            return JSON.parse(result);
        } else {
            clientIcon.classList.add('trigger');
            setTimeout(() => {
                clientIcon.classList.remove('trigger');
            }, 200);

            let result = await this.fragmentExecutor.execute(fragmentId, functionName, parameters);
            return JSON.parse(result);
        }
    }
}
