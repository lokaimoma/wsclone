<script lang="ts">
    import Add from "./svg/Add.svelte";
    import Dashboard from "./svg/Dashboard.svelte";
    import Clones from "./svg/Clone.svelte";
    import Settings from "./svg/Settings.svelte";
    import Logo from "./svg/Logo.svelte";
</script>

<nav class="nav">
    <div class="img-container">
        <Logo />
    </div>
    <ul class="links-container">
        <li><a href="#"><Dashboard /></a></li>
        <li><a href="#"><Clones /></a></li>
        <li><a href="#"><Add fill="red"/></a></li>
        <li><a href="#"><Settings /></li>
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
        background-color: #afafaf;
        width: max-content;
        padding: 1rem;
    }
    .links-container {
        width: max-content;
        list-style: none;
        display: flex;
        flex-direction: column;
        padding: 1rem;
    }
</style>
