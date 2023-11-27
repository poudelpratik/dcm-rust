class InternalEvent extends EventTarget {
    emit(eventName, detail) {
        this.dispatchEvent(new CustomEvent(eventName, { detail }));
    }
}

export const INTERNAL_EVENT_TYPES = {
    EXECUTING_WASM_ON_SERVER: 'EXECUTING_WASM_ON_SERVER',
    EXECUTING_WASM_ON_CLIENT: 'EXECUTING_WASM_ON_CLIENT',
    WASM_RESULT: 'WASM_RESULT',
    UPDATE_FRAGMENTS: 'UPDATE_FRAGMENTS'
}

const internal_event = new InternalEvent();
export default internal_event;
