import { writable } from "svelte/store"

enum Tab {
    DASHBOARD,
    CLONES,
    SETTINGS,
    ADD,
}

function createTabState() {
    const {subscribe, set, update: _} = writable(Tab.DASHBOARD);

    return {
        subscribe,
        setCurrentTab: (tab: Tab) => set(tab)
    }
}

const currentTab = createTabState();

export {
    Tab,
    currentTab
}
