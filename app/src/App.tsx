import ChatContainer from './components/Chat/Container'
import SideBar from './components/Sidebar'

const WS_URL = 'ws://127.0.0.1:9001'

interface NewMessage extends BaseMessage {
    data: {
        userId: string
        roomId: string
        message: string
        serverId: string
    }
}

interface Login extends BaseMessage {
    data: {
        userId: string
    }
}

interface Logout extends BaseMessage {
    data: {
        userId: string
    }
}

interface BaseMessage {
    type: 'newMessage' | 'login' | 'logout'
}

type WSMessage = NewMessage | Login | Logout

const isNewMessage = (message: WSMessage): message is NewMessage =>
    message.type === 'newMessage'
const isLoginMessage = (message: WSMessage): message is NewMessage =>
    message.type === 'login'
const isLogoutMessage = (message: WSMessage): message is NewMessage =>
    message.type === 'logout'

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
