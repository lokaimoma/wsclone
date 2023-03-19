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
