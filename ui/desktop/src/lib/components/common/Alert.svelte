<script lang="ts">
    import type { Alert } from "$lib/types";

    export let alerts: Alert[];
</script>

<div class="container">
    {#each alerts as alert (alert.id)}
        <p>{alert.msg}</p>
    {/each}
</div>

<style>
    .container {
        position: absolute;
        bottom: 0;
        right: 0;
    }
</style>
