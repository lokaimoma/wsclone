import type { Alert } from "$lib/types";
import { writable } from "svelte/store";

const ALERT_TIME_OUT = 3000;

function createAlertState() {
    const { subscribe, set, update } = writable<Alert[]>([]);

    return {
        subscribe,
        add: (alert: Alert) => {
            update((old) => [...old, alert]);
            setTimeout(() => update(old => old.filter(a => a.id !== alert.id)), ALERT_TIME_OUT);
        },
        reset: () => set([]),
        remove: (alertId: Symbol) => update(old => old.filter(alert => alert.id != alertId))
    }
}

const alerts = createAlertState();
export default alerts;

