import { Link, useParams } from 'react-router-dom'
import { mockData } from '../../models/temp'

const SideBar = () => {
    const { userId, serverId } = useParams()

    return (
        <div className="hidden md:flex flex-none flex-col h-full w-48 bg-slate-900 text-white">
            <Link to={``}>Reset</Link>
            {!serverId &&
                mockData.users[userId ?? ''].servers.map((id: any) => (
                    <div key={id} className="w-full my-2">
                        <Link to={`servers/${id}`}>
                            {(mockData as any).servers[id].name}
                        </Link>
                    </div>
                ))}

            {serverId &&
                mockData.servers[serverId].rooms.map((id: any) => (
                    <div key={id} className="w-full my-2">
                        <Link to={`servers/${serverId}/rooms/${id}`}>
                            {(mockData as any).rooms[id].name}
                        </Link>
                    </div>
                ))}
        </div>
    )
}

export default SideBar
