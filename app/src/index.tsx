import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import reportWebVitals from './reportWebVitals'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import ChatContainer from './components/Chat/Container'

const router = createBrowserRouter([
    {
        path: '/',
        element: <App />,
        errorElement: <div>Not found</div>,
        children: [
            {
                path: 'users/:userId/servers/:serverId/rooms/:roomId',
                element: <ChatContainer />,
            },
        ],
    },
])
const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)
root.render(
    <React.StrictMode>
        <RouterProvider router={router} />
    </React.StrictMode>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
