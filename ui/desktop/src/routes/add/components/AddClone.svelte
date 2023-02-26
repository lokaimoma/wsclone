<div class="sec-container">
    <h3>Configure your clone</h3>
    <br>
    <hr />
    <div class="form">
        <label>
            <span>Title</span>
            <input type="text" />
        </label>
        <label>
            <span>Description</span>
            <textarea rows="3" />
        </label>
        <label>
            <span>URL</span>
            <input type="url" />
        </label>
        <label>
            <span>Max level</span>
            <select>
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
            <select>
                <option value="10 mb">10 mb</option>
                <option value="20 mb">20 mb</option>
                <option value="30 mb">30 mb</option>
            </select>
        </label>
        <label>
            <span>Blacklist url/patterns</span>
            <textarea rows="3" />
        </label>
    </div>
    <button disabled>Proceed</button>
</div>

<style>
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

    input:focus,textarea:focus,select:focus {
        border-color: #333;
    }

    select {
        min-width: max-content;
    }

    button {
        color: #fff;
        background-color: #333;
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
        background-color: #858585;
        color: rgba(255, 255, 255, 0.7);
    }
</style>
