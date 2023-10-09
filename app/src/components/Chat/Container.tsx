import { useEffect, useRef, useState } from 'react'
import { NewMessage, WSMessage } from '../../models/websocket'
import MessageBubble from './MessageBubble'
import SendMessage from './SendMessage'

const WS_URL = 'ws://127.0.0.1:9001'
const ChatContainer = () => {
    const socket = useRef<WebSocket | null>(null)
    const [messages, setMessages] = useState<WSMessage[]>([])

    const handleSendMessage = async (message: string) => {
        if (socket.current?.readyState === WebSocket.OPEN) {
            const formattedMessage = new NewMessage({
                userId: '6ec4844c-9ccc-4fe3-9ad6-86377ba3448b',
                roomId: '0e67e2e4-cb84-46a8-a209-362a7d50f620',
                message,
                serverId: '0e67e2e4-cb84-46a8-a209-362a7d50f620',
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
            setMessages((m) => [...m, JSON.parse(event.data)])
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
