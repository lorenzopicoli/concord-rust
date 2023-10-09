import { useState, KeyboardEvent } from 'react'

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

export default SendMessage
