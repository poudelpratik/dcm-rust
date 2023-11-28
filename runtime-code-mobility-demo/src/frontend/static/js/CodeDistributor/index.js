import WebSocketClient, {MESSAGE_TYPES} from "./websocketclient.js";
import FragmentRegistry from "./fragment-registry.js";
import FragmentExecutor from "./fragment-executor.js";
import ExecutionLocationSelector from "./execution-location-selector.js";
import {EXECUTION_LOCATION} from "./constants.js";

export default class CodeDistributionManager {

    constructor(configuration) {
        this.configuration = configuration;
        this.webSocketClient = new WebSocketClient(configuration);
        this.fragmentRegistry = new FragmentRegistry(configuration);
    }

    async init() {
        await this.fragmentRegistry.init();
        await this.webSocketClient.init(this.fragmentRegistry);
        this.fragmentExecutor = new FragmentExecutor(this.fragmentRegistry, this.configuration);
        await this.fragmentExecutor.init();
        new ExecutionLocationSelector(this);
    }

    async updateFragmentRegistry(fragmentId, executionLocation) {
        this.fragmentRegistry.update(fragmentId, executionLocation);
        const connectionIcon = document.querySelector('#connection-icon');
        connectionIcon.classList.add('trigger');
        setTimeout(() => {
            connectionIcon.classList.remove('trigger');
        }, 200);
        await this.webSocketClient.sendMessage(MESSAGE_TYPES.UPDATE_FRAGMENTS, [{
            fragment_id: fragmentId,
            execution_location: executionLocation
        }]);
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
