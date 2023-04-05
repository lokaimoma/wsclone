import { invoke } from "@tauri-apps/api";
import type { CloneProp, Command } from "./types";
import { CommandType } from "./types";

enum Commands {
    HANDLE_COMMAND = "handle_command"
}

class Psychic {
    public static async clone_site(props: CloneProp): Promise<void> {
        let cmd: Command = { type: CommandType.CLONE, props: JSON.stringify(props), keepAlive: true };
        await invoke<void>(`plugin:wsclone|${Commands.HANDLE_COMMAND}`, { jsonPayload: cmd });
    }
}

export default Psychic;
