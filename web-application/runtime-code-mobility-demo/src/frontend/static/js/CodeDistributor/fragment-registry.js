import internal_event, {INTERNAL_EVENT_TYPES} from "./internal-events.js";

export default class FragmentRegistry {
    fragmentMap = new Map();

    constructor(configuration) {
        this.configuration = configuration;
    }

    async init() {
        await this.fetchFragments();
        internal_event.addEventListener(INTERNAL_EVENT_TYPES.UPDATE_FRAGMENTS, (event) => {
            event.detail.data.forEach((fragment) => {
                if (this.fragmentMap.has(fragment.id)) {
                    console.log(`Updating fragment ${fragment.id} to ${fragment.execution_location}`);
                    this.fragmentMap.set(fragment.id, fragment.execution_location);
                }
            });
        });
    }

    async fetchFragments() {
        const response = await fetch(`${this.configuration.codeDistributorDir}fragments/executable_fragments.json`);
        const fragments = await response.json();
        this.fragmentMap = new Map(fragments.map(obj => [obj.id, obj.execution_location]));
    }

    update(fragmentId, executionLocation) {
        console.log(`Updating fragment ${fragmentId} to ${executionLocation}`);
        this.fragmentMap.set(fragmentId, executionLocation);
    }
}
