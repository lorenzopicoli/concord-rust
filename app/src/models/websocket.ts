import { ChatMessage } from './chat'

export enum WSMessageType {
    NewMessage = 'newMessage',
    Login = 'login',
    Logout = 'logout',
}

export class GenericMessage {
    public type: WSMessageType
    public data: unknown

    public toChatMessage(): ChatMessage {
        return { message: 'Uknown message', username: 'System' }
    }

    constructor(rawMessage: Record<string, unknown>) {
        if (!rawMessage.type) {
            throw new Error("Can't create ws message without a type")
        }

        // Assume type is of a message type, try to match it to a type defined
        const type = rawMessage.type as WSMessageType
        if (
            Object.values(WSMessageType).indexOf(type as WSMessageType) === -1
        ) {
            throw new Error('Invalid message type')
        }
        this.type = type

        this.data = rawMessage.data
    }

    public isNewMessage(): this is NewMessage {
        return this.type === WSMessageType.NewMessage
    }

    public isLoginMessage(): this is NewMessage {
        return this.type === WSMessageType.Login
    }

    public isLogoutMessage(): this is NewMessage {
        return this.type === WSMessageType.Logout
    }
}

export class NewMessage extends GenericMessage {
    public data: {
        userId: string
        roomId: string
        message: string
        serverId: string
    }

    constructor(data: NewMessage['data']) {
        super({ type: WSMessageType.NewMessage })
        this.data = data
    }

    public toChatMessage(): ChatMessage {
        return {
            message: this.data.message,
            username: this.data.userId,
        }
    }
}

export class LoginMessage extends GenericMessage {
    public data: {
        userId: string
    }
    constructor(data: LoginMessage['data']) {
        super({ type: WSMessageType.Login })
        this.data = data
    }

    public toChatMessage(): ChatMessage {
        return {
            message: `User logged in ${this.data.userId}`,
            username: 'System',
        }
    }
}

export class LogoutMessage extends GenericMessage {
    public data: {
        userId: string
    }

    constructor(data: LogoutMessage['data']) {
        super({ type: WSMessageType.Logout })
        this.data = data
    }

    public toChatMessage(): ChatMessage {
        return {
            message: `User logged out ${this.data.userId}`,
            username: this.data.userId,
        }
    }
}

export const parseWSMessage = (
    rawMessage: Record<string, unknown>
): GenericMessage => {
    if (!rawMessage.type) {
        throw new Error("Can't create ws message without a type")
    }

    // Assume type is of a message type, try to match it to a type defined
    const type = rawMessage.type as WSMessageType
    if (Object.values(WSMessageType).indexOf(type as WSMessageType) === -1) {
        throw new Error('Invalid message type')
    }
    switch (type) {
        //TODO: Remove any
        case WSMessageType.NewMessage:
            return new NewMessage(rawMessage.data as any)
        case WSMessageType.Login:
            return new LoginMessage(rawMessage.data as any)
        case WSMessageType.Logout:
            return new LogoutMessage(rawMessage.data as any)
    }
}
