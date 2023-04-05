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

export type Command = {
    type: CommandType,
    props: string,
    keepAlive: boolean | null
}

export enum CommandType {
    CLONE = "CLONE",
    HEALTH_CHECK = "HEALTH_CHECK",
    GET_CLONES = "GET_CLONES",
    ABORT_CLONE = "ABORT_CLONE",
    CLONE_STATUS = "CLONE_STATUS",
}

export type CloneProp = {
    sessionId: string,
    link: string,
    dirName: string,
    maxStaticFileSize: number,
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
