import { useEffect, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import {
    GenericMessage,
    LoginMessage,
    NewMessage,
    parseWSMessage,
} from '../../models/websocket'
import MessageBubble from './MessageBubble'
import SendMessage from './SendMessage'

const WS_URL = 'ws://127.0.0.1:9001'
const ChatContainer = () => {
    const socket = useRef<WebSocket | null>(null)
    const [messages, setMessages] = useState<GenericMessage[]>([])
    const { userId, roomId, serverId } = useParams()

    const handleSendMessage = async (message: string) => {
        if (socket.current?.readyState === WebSocket.OPEN) {
            const formattedMessage = new NewMessage({
                userId: userId ?? '',
                roomId: roomId ?? '',
                message,
                serverId: serverId ?? '',
            })

            socket.current.send(JSON.stringify(formattedMessage))
        } else {
            console.log('WebSocket connection not open.')
            throw new Error('Not connected')
        }
    }

    useEffect(() => {
        socket.current = new WebSocket(WS_URL)

        socket.current.onmessage = (event) => {
            const message = parseWSMessage(JSON.parse(event.data))
            console.log('New message', message)
            setMessages((m) => [...m, message])
        }

        const loginMessage = new LoginMessage({
            userId: userId ?? '',
        })

        socket.current.onopen = () => {
            socket.current?.send(JSON.stringify(loginMessage))
        }

        // Clean up the WebSocket connection on component unmount
        return () => {
            if (socket.current) {
                socket.current.close()
            }
        }
    }, []) // Empty dependency array ensures this effect runs once after the initial render

    return (
        <div className="flex-1 flex flex-col justify-end overflow-y-auto">
            {messages.map((message) => (
                <>
                    <MessageBubble message={message.toChatMessage()} />
                </>
            ))}
            <SendMessage onSend={handleSendMessage} />
        </div>
    )
}

export default ChatContainer
