<script lang="ts">
    import { onMount } from "svelte";
    import { currentTab, Tab } from "$lib/stores/Navbar";
    import AddClone from "./components/AddClone.svelte";

    onMount(() => {
        currentTab.setCurrentTab(Tab.ADD);
    });
</script>

<div class="page">
   <AddClone /> 
</div>
