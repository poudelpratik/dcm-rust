export default class FragmentRegistry {
    fragmentMap = new Map();

    constructor(configuration) {
        this.configuration = configuration;
        window.addEventListener('beforeunload', () => {
            localStorage.setItem('fragmentRegistry', JSON.stringify(Array.from(this.fragmentMap, ([key, value]) => ({
                ['id']: key,
                ['execution_location']: value
            }))));
        });
    }

    async init() {
        await this.fetchFragments();
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
