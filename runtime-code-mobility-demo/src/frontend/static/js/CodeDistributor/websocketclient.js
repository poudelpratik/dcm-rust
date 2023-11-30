import internal_event, {INTERNAL_EVENT_TYPES} from "./internal-events.js";
import {isUndefined, selectElement, uuidv4} from "./utils.js";

export class MESSAGE_TYPES {
    static EXECUTE_FUNCTION = 'ExecuteFunction';
    static WASM_RESULT = 'WasmResult';
    static UPDATE_FRAGMENTS = 'UpdateFragments';
}


export default class WebSocketClient {

    /** @type {WebSocket} */
    #accessToken = '';

    // reconnect measures
    #reconnectTimer = 5; // seconds
    #maxReconnectAttempts = 3;
    #reconnectAttempts = 0;
    #reconnectTimerMs = this.#reconnectTimer * 1000;

    #globalReconnectTimer = this.#reconnectTimer * this.#maxReconnectAttempts;
    #globalReconnectIntervalTimer = undefined;

    #reconnectCounterElement = selectElement('#connection-icon');


    constructor(config) {
        this.socket = null;
        this.pendingRequests = {};
        this.config = config;
        this.#maxReconnectAttempts = config.maxReconnectAttempts;
    }

    async init(auth) {
        this.auth = auth;
        await this.establishConnection();
    }

    async establishConnection() {
        this.socket = new WebSocket(this.config.codeDistributorWsUrl + '?auth_token=' + this.auth.token);
        this.socket.onopen = () => this.onOpen();
        this.socket.onmessage = (event) => this.onMessage(event);
        this.socket.onclose = () => {
            this.onClose();
            this.attemptReconnect();
        };
    }

    onMessage(event) {
        const message = JSON.parse(event.data);
        switch (message.message_type) {
            case MESSAGE_TYPES.EXECUTE_FUNCTION:
                if (this.pendingRequests[message.message_id]) {
                    this.pendingRequests[message.message_id].resolve(message.data);
                    delete this.pendingRequests[message.message_id];
                }
                break;
            case MESSAGE_TYPES.UPDATE_FRAGMENTS:
                internal_event.emit(INTERNAL_EVENT_TYPES.UPDATE_FRAGMENTS, message);
        }
    }

    onOpen() {
        this.#reconnectAttempts = 0;
        clearInterval(this.#globalReconnectIntervalTimer);
        const websocketStatus = selectElement('#websocketStatus');
        !isUndefined(websocketStatus)
            ? (websocketStatus.dataset.status = 'online')
            : null;

        const connectionIcon = document.querySelector('#connection-icon');
        connectionIcon.classList.add('trigger');
        setTimeout(() => {
            connectionIcon.classList.remove('trigger');
        }, 200);
        console.log('WebSocket Client Connected');
    }

    onClose() {
        const websocketStatus = selectElement('#websocketStatus');
        !isUndefined(websocketStatus)
            ? (websocketStatus.dataset.status = 'offline')
            : null;
        console.log('WebSocket Connection Closed');
    }

    sendMessage(eventType, data) {
        if (this.socket.readyState === WebSocket.OPEN) {
            return new Promise((resolve, reject) => {
                const message_id = uuidv4();
                this.pendingRequests[message_id] = {resolve, reject};
                const message = {
                    message_id: message_id,
                    message_type: eventType,
                    data: data,
                }
                this.socket.send(JSON.stringify(message));
            });
        } else {
            console.error('WebSocket is not open:', this.socket.readyState);
        }
    }

    isOpen() {
        return this.socket.readyState === WebSocket.OPEN;
    }

    attemptReconnect() {
        if (this.#reconnectAttempts < this.#maxReconnectAttempts) {
            // Clear any existing interval timer
            clearInterval(this.#globalReconnectIntervalTimer);

            // Set timeout for next reconnect attempt
            setTimeout(() => {
                console.log(`Attempting to reconnect... (Attempt ${this.#reconnectAttempts + 1} of ${this.#maxReconnectAttempts})`);

                let remainingTime = this.#reconnectTimer;
                this.#globalReconnectIntervalTimer = setInterval(() => {
                    remainingTime--;
                    this.#reconnectCounterElement.setAttribute('style', `--content:'${remainingTime}'`);

                    // Clear interval when countdown reaches zero
                    if (remainingTime <= 0) {
                        clearInterval(this.#globalReconnectIntervalTimer);
                    }
                }, 1000);

                this.#reconnectAttempts++;
                this.establishConnection();
            }, this.#reconnectTimerMs);
        } else {
            console.log("Max reconnect attempts reached. No longer trying to reconnect.");
            clearInterval(this.#globalReconnectIntervalTimer);
        }
    }
};
