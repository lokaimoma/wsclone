<script lang="ts">
    import "../app.css";
    import Navbar from "$lib/components/common/Navbar.svelte";
</script>

<div class="pane">
    <Navbar />
    <slot />
</div>

<style>
    .pane {
        display: grid;
        grid-template-columns: auto 1fr;
    }

    :global(.page) {
        padding: 1rem;
    }
</style>
