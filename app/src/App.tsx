import React, { useEffect, useRef, useState, KeyboardEvent } from 'react'

const WS_URL = 'ws://127.0.0.1:9001'

const SideBar = () => {
    return (
        <div className="hidden md:flex flex-none h-full w-48 bg-slate-900 text-white">
            <>asad</>
        </div>
    )
}
interface MessageBubbleProps {
    message: string
    username: string
}
const MessageBubble = (props: MessageBubbleProps) => {
    return (
        <div className="flex flex-row px-4 py-2 hover:bg-slate-900">
            <img
                src="https://upload.wikimedia.org/wikipedia/commons/2/2c/Default_pfp.svg"
                alt="Profile"
                className="w-9 h-9 rounded-full mr-1 mt-1"
            />
            <div className="flex-1 ml-3 flex flex-col">
                <div className="text-white font-semibold flex-1 max-w-xs leading-tight mb-1">
                    {props.username}
                </div>
                <div className="text-white flex-1">{props.message}</div>
            </div>
        </div>
    )
}
interface SendMessageProps {
    onSend: (message: string) => Promise<void>
}
const SendMessage = (props: SendMessageProps) => {
    const [messageBoxValue, setMessageBoxValue] = useState('') // State to store the input value
    const handleMessageBoxValueChange = (e: any) => {
        setMessageBoxValue(e.target.value)
    }
    const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
        if (event.key === 'Enter') {
            handleSendMessage()
        }
    }
    const handleSendMessage = async () => {
        await props
            .onSend(messageBoxValue)
            .then(() => setMessageBoxValue(''))
            .catch(() => '')
    }
    return (
        <div className="flex items-center p-4 mt-4">
            <input
                type="text"
                placeholder="Type your message..."
                value={messageBoxValue}
                onChange={handleMessageBoxValueChange}
                onKeyDown={handleKeyDown}
                className="flex-1 rounded-l-lg h-12 bg-slate-900 py-2 px-4 focus:outline-none"
            />

            <button
                onClick={handleSendMessage}
                className="text-white h-12 rounded-r-lg bg-slate-900 px-4 py-2 focus:outline-none hover:bg-slate-800"
            >
                Send
            </button>
        </div>
    )
}

const getRandomUsername = (): string => {
    const animalNames: string[] = [
        'Lion',
        'Tiger',
        'Elephant',
        'Giraffe',
        'Monkey',
        'Kangaroo',
        'Zebra',
        'Hippopotamus',
        'Panda',
        'Koala',
        'Dolphin',
        'Penguin',
        'Eagle',
        'Ostrich',
        'Crocodile',
        'Gorilla',
    ]

    const randomIndex: number = Math.floor(Math.random() * animalNames.length)
    const randomAnimalName: string = animalNames[randomIndex]
    return randomAnimalName
}

const ChatContainer = () => {
    const socket = useRef<WebSocket | null>(null)
    const [messages, setMessages] = useState<string[]>([])

    const handleSendMessage = async (message: string) => {
        if (socket.current?.readyState === WebSocket.OPEN) {
            socket.current.send(message)
        } else {
            console.log('WebSocket connection not open.')
            throw new Error('Not connected')
        }
    }

    useEffect(() => {
        socket.current = new WebSocket(WS_URL)

        socket.current.onmessage = (event) => {
            setMessages((m) => [...m, JSON.parse(event.data).data as string])
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
                    <MessageBubble
                        username={getRandomUsername()}
                        message={message}
                    />
                </>
            ))}
            <SendMessage onSend={handleSendMessage} />
        </div>
    )
}

function App() {
    return (
        <div className="flex h-screen bg-slate-950 text-white">
            <SideBar />
            <div className="flex flex-1">
                <ChatContainer />
            </div>
        </div>
    )
}

export default App
