import { useState, useEffect } from 'react'
import { Routes, Route } from 'react-router-dom'
import Layout from './components/layout'
import Dashboard from './pages/dashboard'
import Groups from './pages/groups'
import Rules from './pages/rules'
import History from './pages/history'
import Login from './pages/login'
import Projects from './pages/projects'

export default function App() {
  const [user, setUser] = useState<{ username: string; token: string } | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const savedUser = localStorage.getItem('admin_user')
    const savedToken = localStorage.getItem('admin_token')
    if (savedUser && savedToken) {
      setUser({ username: savedUser, token: savedToken })
    }
    setIsLoading(false)
  }, [])

  const handleLogin = (username: string, token: string) => {
    localStorage.setItem('admin_user', username)
    localStorage.setItem('admin_token', token)
    setUser({ username, token })
  }

  const handleLogout = () => {
    localStorage.removeItem('admin_user')
    localStorage.removeItem('admin_token')
    setUser(null)
  }

  if (isLoading) {
    return null
  }

  if (!user) {
    return <Login onLogin={handleLogin} />
  }

  return (
    <Routes>
      <Route path="/" element={<Layout user={user} onLogout={handleLogout} />}>
        <Route index element={<Dashboard />} />
        <Route path="groups" element={<Groups />} />
        <Route path="rules" element={<Rules />} />
        <Route path="projects" element={<Projects />} />
        <Route path="history" element={<History />} />
      </Route>
    </Routes>
  )
}
