import { ChatMessage } from './chat'

export enum WSMessageType {
    NewMessage = 'newMessage',
    Login = 'login',
    Logout = 'logout',
}

export class BaseMessage {
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

export class NewMessage extends BaseMessage {
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

export class LoginMessage extends BaseMessage {
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

export class LogoutMessage extends BaseMessage {
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

export type WSMessage = NewMessage | LoginMessage | LogoutMessage
