<script lang="ts">
    let description: string;
    let title: string;
    let url: URL;
    let maxLevel: number;
    let maxFileSize: number;
    let blackListUrls: string;
    let urlError = false;
    let processingReq = false;

    //validate url and set
    function onUrlChange(event: Event) {
        const target = event.target as HTMLInputElement;
        urlError = false;
        try {
            url = new URL(target.value);
        } catch (e) {
            urlError = true;
        }
    }

    function onProceedBtnClicked() {
        processingReq = true;
    }
</script>

<div class="sec-container">
    <h3>Configure your clone</h3>
    <br />
    <hr />
    <form class="form">
        <label>
            <span>Title</span>
            <input type="text" bind:value={title} required />
        </label>
        <label>
            <span>Description</span>
            <textarea rows="3" bind:value={description} />
        </label>
        <label>
            <span>URL</span>
            <div>
                <input
                    type="url"
                    on:change={(e) => onUrlChange(e)}
                    required
                    placeholder="https://somedomain.com/"
                />
                {#if urlError}
                    <p class="error">Invalid url</p>
                {/if}
            </div>
        </label>
        <label>
            <span>Max level</span>
            <select bind:value={maxLevel}>
                <option value="0">0</option>
                <option value="1">1</option>
                <option value="2">2</option>
                <option value="3">3</option>
                <option value="4">4</option>
                <option value="5">5</option>
                <option value="6">6</option>
                <option value="7">7</option>
                <option value="8">8</option>
                <option value="9">9</option>
            </select>
        </label>
        <label
            ><span>Max file size</span>
            <select bind:value={maxFileSize}>
                <option value="10 mb">10 mb</option>
                <option value="20 mb">20 mb</option>
                <option value="30 mb">30 mb</option>
            </select>
        </label>
        <label>
            <span>Blacklist url/patterns</span>
            <textarea rows="3" bind:value={blackListUrls} />
        </label>
    </form>
    {#if !processingReq}
        <button
            on:click={onProceedBtnClicked}
            disabled={url === undefined || (title?.length ?? 0) == 0}
            >Proceed</button
        >
    {:else}
        <button disabled>
            <div>
                Please wait<span class="inc-disp"
                    ><div class="hider" />
                    ...</span
                >
            </div>
        </button>
    {/if}
</div>

<style>
    :root {
        --text-sm: 0.7em;
        --clr-disabled: #858585;
        --clr-secondary: #333;
    }

    .sec-container {
        position: relative;
    }

    .form {
        display: flex;
        flex-direction: column;
        gap: 1.4rem;
        padding: 1rem;
    }

    label {
        display: flex;
        align-items: center;
    }

    label > *:first-child {
        min-width: 20ch;
        width: max-content;
    }

    input,
    select,
    textarea {
        outline: none;
        border: 2px solid #ccc;
        border-radius: 5px;
        padding: 0.5rem 0.8rem;
        min-width: 50ch;
        resize: none;
        transition: border-color 250ms ease-in-out;
    }

    input:focus,
    textarea:focus,
    select:focus {
        border-color: #333;
    }

    select {
        min-width: max-content;
    }

    button {
        color: #fff;
        background-color: var(--clr-secondary);
        padding: 0.3rem 1rem;
        border: none;
        font-weight: bold;
        border-radius: 5px;
        position: absolute;
        right: 0;
        margin-block-start: 0.9rem;
        cursor: pointer;
    }

    button:enabled {
        box-shadow: 0 5px 10px rgba(0, 0, 0, 0.2);
    }

    button:active {
        box-shadow: none;
    }

    button:disabled {
        background-color: var(--clr-disabled);
        color: rgba(255, 255, 255, 0.7);
        cursor: initial;
    }
    .error {
        color: red;
        font-weight: bold;
        font-size: var(--text-sm);
    }

    .inc-disp {
        position: relative;
        display: inline-block;
    }

    .hider {
        background: var(--clr-disabled);
        z-index: 2;
        position: absolute;
        inset: 0;
        animation: move-right 3s ease-in-out alternate infinite both;
    }

    @keyframes move-right {
        0% {
            left: 0px;
        }

        100% {
            left: 11px;
        }
    }
</style>
