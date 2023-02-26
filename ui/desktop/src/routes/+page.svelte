<script lang="ts">
    import { onMount } from "svelte";
    import { currentTab, Tab } from "$lib/stores/Navbar";
    onMount(() => {
        currentTab.setCurrentTab(Tab.DASHBOARD);
    });
</script>

<div class="page">
    <h1>Dashboard</h1>
    <p>
        Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint
        cillum sint consectetur cupidatat.
    </p>
</div>
