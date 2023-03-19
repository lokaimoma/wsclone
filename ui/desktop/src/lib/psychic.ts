import { invoke } from "@tauri-apps/api";

enum Commands {
    HANDLE_COMMAND = "handle_command"
}

class Psychic {
    public static async clone_site(): Promise<void> {
        await invoke<void>(`plugin:wsclone|${Commands.HANDLE_COMMAND}`);
    }
}

export default Psychic;
