import { Link } from 'react-router-dom'
import { mockData } from '../../models/temp'

const Login = () => {
    return (
        <div className="hidden md:flex flex-none h-full w-48 bg-slate-900 text-white">
            {Object.keys(mockData.users).map((id) => (
                <Link to={`/users/${id}`}>
                    {(mockData as any).users[id].username}
                </Link>
            ))}
        </div>
    )
}

export default Login
