import { ChatMessage } from '../../models/chat'

interface MessageBubbleProps {
    message: ChatMessage
}
const MessageBubble = ({ message }: MessageBubbleProps) => {
    return (
        <div className="flex flex-row px-4 py-2 hover:bg-slate-900">
            <img
                src="https://upload.wikimedia.org/wikipedia/commons/2/2c/Default_pfp.svg"
                alt="Profile"
                className="w-9 h-9 rounded-full mr-1 mt-1"
            />
            <div className="flex-1 ml-3 flex flex-col">
                <div className="text-white font-semibold flex-1 max-w-xs leading-tight mb-1">
                    {message.username}
                </div>
                <div className="text-white flex-1">{message.message}</div>
            </div>
        </div>
    )
}

export default MessageBubble
