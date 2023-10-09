import { Outlet } from 'react-router-dom'
import SideBar from './components/Sidebar'

function App() {
    return (
        <div className="flex h-screen bg-slate-950 text-white">
            <SideBar />
            <div className="flex flex-1">
                <Outlet />
            </div>
        </div>
    )
}

export default App
