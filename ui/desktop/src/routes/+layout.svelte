<script lang="ts">
    import "../app.css";
    import Navbar from "$lib/components/common/Navbar.svelte";
    import Alert from "$lib/components/common/Alert.svelte";
    import Alerts from "$lib/stores/Alert";
</script>

<div class="pane">
    <Navbar />
    <slot />
    <Alert alerts={$Alerts}/>
</div>

<style>
    .pane {
        display: grid;
        grid-template-columns: auto 1fr;
    }

    :global(.page) {
        padding: 1rem;
        background-color: #F0F0F0;
    }
</style>
