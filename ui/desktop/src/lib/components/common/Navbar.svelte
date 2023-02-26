<script lang="ts">
    import Add from "./svg/Add.svelte";
    import Dashboard from "./svg/Dashboard.svelte";
    import Clones from "./svg/Clone.svelte";
    import Settings from "./svg/Settings.svelte";
    import Logo from "./svg/Logo.svelte";
    import { Tab, currentTab } from "$lib/stores/Navbar";
</script>

<nav class="nav">
    <div class="img-container">
        <Logo />
    </div>
    <ul class="links-container">
        <li class={$currentTab === Tab.DASHBOARD ? "current" : ""}>
            <a href="/"
                ><Dashboard
                    fill={$currentTab === Tab.DASHBOARD ? "#ccc" : "#333"}
                />
            </a>
        </li>
        <li class={$currentTab === Tab.CLONES ? "current" : ""}>
            <a href="/clones"
                ><Clones fill={$currentTab === Tab.CLONES ? "#ccc" : "#333"} />
            </a>
        </li>
        <li class={$currentTab === Tab.ADD ? "current" : ""}>
            <a href="/add"
                ><Add fill={$currentTab === Tab.ADD ? "#ccc" : "#333"} />
            </a>
        </li>
        <li class={$currentTab === Tab.SETTINGS ? "current" : ""}>
            <a href="/settings"
                ><Settings
                    fill={$currentTab === Tab.SETTINGS ? "#ccc" : "#333"}
                />
            </a>
        </li>
    </ul>
</nav>

<style>
    .nav {
        height: 100vh;
        display: flex;
        flex-direction: column;
        background-color: #fff;
        box-shadow: 2px 0 10px rgba(0, 0, 0, 0.04);
        width: max-content;
    }
    .img-container {
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: #545050;
        width: max-content;
    }
    .links-container {
        width: max-content;
        list-style: none;
        display: flex;
        flex-direction: column;
    }
    .img-container,
    li {
        padding: 1rem;
    }

    li.current {
        background-color: #333;
    }
</style>
