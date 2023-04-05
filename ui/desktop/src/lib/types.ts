export type Alert = {
    id: Symbol,
    msg: string,
    icon: AlertIcon,
}

export enum AlertIcon {
    ERROR,
    SUCCESS,
    DEFAULT,
}

export type Command<F> = {
    type: CommandType,
    props: F,
    keep_alive: boolean | null
}

export enum CommandType {
    CLONE,
    HEALTH_CHECK,
    GET_CLONES,
    ABORT_CLONE,
    CLONE_STATUS,
}

export type CloneProp = {
    sessionId: string,
    link: string,
    dirName: string,
    maxStaticFileSize: string,
    downloadStaticResourceWithUnknownSize: boolean,
    progressUpdateInterval: number,
    maxLevel: number,
    blackListUrls: string[],
    abortOnDownloadError: boolean
}

export type AbortCloneProp = {
    sessionId: string
}

export type CloneStatusProp = {
    sessionId: string
}
