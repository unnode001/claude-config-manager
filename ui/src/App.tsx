import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import './App.css'

// Types
interface McpServer {
  name: string
  enabled: boolean
  command: string
  args: string[]
  env: Record<string, string>
}

interface ClaudeConfig {
  mcp_servers?: Record<string, McpServer>
  skills?: Record<string, unknown>
  allowed_paths?: string[]
  custom_instructions?: string[]
  unknown: Record<string, unknown>
}

interface Project {
  name: string
  path: string
  root: string
  claude_dir: string
  has_config: boolean
}

interface Backup {
  path: string
  original_path: string
  created_at: string
  size: number
}

type Tab = 'config' | 'mcp' | 'projects' | 'history'

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('config')
  const [config, setConfig] = useState<ClaudeConfig | null>(null)
  const [projects, setProjects] = useState<Project[]>([])
  const [backups, setBackups] = useState<Backup[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load initial data
  useEffect(() => {
    loadConfig()
    loadProjects()
    loadBackups()
  }, [])

  const loadConfig = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<ClaudeConfig>('get_config', { projectPath: null })
      setConfig(result)
    } catch (err) {
      setError(err as string)
    } finally {
      setLoading(false)
    }
  }

  const loadProjects = async () => {
    try {
      const result = await invoke<Project[]>('list_projects')
      setProjects(result)
    } catch (err) {
      console.error('Failed to load projects:', err)
    }
  }

  const loadBackups = async () => {
    try {
      const result = await invoke<Backup[]>('list_backups')
      setBackups(result)
    } catch (err) {
      console.error('Failed to load backups:', err)
    }
  }

  const openConfigFile = async () => {
    try {
      const path = await invoke<string>('get_global_config_path')
      await open(path)
    } catch (err) {
      setError(err as string)
    }
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Claude Config Manager</h1>
        <div className="header-actions">
          <button onClick={openConfigFile} className="btn-secondary">
            Open Config File
          </button>
          <button onClick={loadConfig} className="btn-secondary" disabled={loading}>
            Refresh
          </button>
        </div>
      </header>

      <main className="app-main">
        <nav className="sidebar">
          <button
            className={`nav-btn ${activeTab === 'config' ? 'active' : ''}`}
            onClick={() => setActiveTab('config')}
          >
            Configuration
          </button>
          <button
            className={`nav-btn ${activeTab === 'mcp' ? 'active' : ''}`}
            onClick={() => setActiveTab('mcp')}
          >
            MCP Servers
          </button>
          <button
            className={`nav-btn ${activeTab === 'projects' ? 'active' : ''}`}
            onClick={() => setActiveTab('projects')}
          >
            Projects
          </button>
          <button
            className={`nav-btn ${activeTab === 'history' ? 'active' : ''}`}
            onClick={() => setActiveTab('history')}
          >
            History
          </button>
        </nav>

        <section className="content">
          {error && <div className="error-banner">{error}</div>}

          {activeTab === 'config' && (
            <ConfigView config={config} onRefresh={loadConfig} loading={loading} />
          )}

          {activeTab === 'mcp' && (
            <McpView config={config} onRefresh={loadConfig} loading={loading} />
          )}

          {activeTab === 'projects' && (
            <ProjectsView projects={projects} onRefresh={loadProjects} loading={loading} />
          )}

          {activeTab === 'history' && (
            <HistoryView backups={backups} onRefresh={loadBackups} loading={loading} />
          )}
        </section>
      </main>
    </div>
  )
}

// Configuration View Component
function ConfigView({ config, onRefresh, loading }: {
  config: ClaudeConfig | null
  onRefresh: () => void
  loading: boolean
}) {
  if (loading) return <div className="loading">Loading...</div>
  if (!config) return <div className="empty-state">No configuration found</div>

  return (
    <div className="view">
      <h2>Configuration</h2>

      <div className="section">
        <h3>Allowed Paths</h3>
        {config.allowed_paths?.length ? (
          <ul className="list">
            {config.allowed_paths.map((path, i) => (
              <li key={i}>{path}</li>
            ))}
          </ul>
        ) : (
          <p className="empty">No allowed paths configured</p>
        )}
      </div>

      <div className="section">
        <h3>Custom Instructions</h3>
        {config.custom_instructions?.length ? (
          <ul className="list">
            {config.custom_instructions.map((instruction, i) => (
              <li key={i}>{instruction}</li>
            ))}
          </ul>
        ) : (
          <p className="empty">No custom instructions configured</p>
        )}
      </div>

      <button onClick={onRefresh} className="btn-primary">
        Refresh
      </button>
    </div>
  )
}

// MCP Servers View Component
function McpView({ config, onRefresh, loading }: {
  config: ClaudeConfig | null
  onRefresh: () => void
  loading: boolean
}) {
  const [newServer, setNewServer] = useState({ name: '', command: '', args: '' })
  const [adding, setAdding] = useState(false)

  const servers = config?.mcp_servers || {}

  const handleAdd = async () => {
    if (!newServer.name || !newServer.command) return

    setAdding(true)
    try {
      await invoke('add_server', {
        name: newServer.name,
        command: newServer.command,
        args: newServer.args ? newServer.args.split(' ') : []
      })
      setNewServer({ name: '', command: '', args: '' })
      onRefresh()
    } catch (err) {
      alert('Failed to add server: ' + err)
    } finally {
      setAdding(false)
    }
  }

  const handleToggle = async (name: string, enabled: boolean) => {
    try {
      if (enabled) {
        await invoke('enable_server', { name })
      } else {
        await invoke('disable_server', { name })
      }
      onRefresh()
    } catch (err) {
      alert('Failed to toggle server: ' + err)
    }
  }

  const handleRemove = async (name: string) => {
    if (!confirm(`Remove server "${name}"?`)) return

    try {
      await invoke('remove_server', { name })
      onRefresh()
    } catch (err) {
      alert('Failed to remove server: ' + err)
    }
  }

  if (loading) return <div className="loading">Loading...</div>

  return (
    <div className="view">
      <h2>MCP Servers</h2>

      <div className="add-server-form">
        <input
          type="text"
          placeholder="Server name"
          value={newServer.name}
          onChange={(e) => setNewServer({ ...newServer, name: e.target.value })}
        />
        <input
          type="text"
          placeholder="Command (e.g., npx)"
          value={newServer.command}
          onChange={(e) => setNewServer({ ...newServer, command: e.target.value })}
        />
        <input
          type="text"
          placeholder="Args (space-separated)"
          value={newServer.args}
          onChange={(e) => setNewServer({ ...newServer, args: e.target.value })}
        />
        <button onClick={handleAdd} disabled={adding} className="btn-primary">
          {adding ? 'Adding...' : 'Add Server'}
        </button>
      </div>

      <div className="server-list">
        {Object.entries(servers).map(([name, server]) => (
          <div key={name} className="server-item">
            <div className="server-info">
              <span className={`status ${server.enabled ? 'enabled' : 'disabled'}`}>
                {server.enabled ? '●' : '○'}
              </span>
              <span className="name">{name}</span>
              <code className="command">{server.command} {server.args.join(' ')}</code>
            </div>
            <div className="server-actions">
              <button
                onClick={() => handleToggle(name, !server.enabled)}
                className="btn-small"
              >
                {server.enabled ? 'Disable' : 'Enable'}
              </button>
              <button
                onClick={() => handleRemove(name)}
                className="btn-small btn-danger"
              >
                Remove
              </button>
            </div>
          </div>
        ))}
      </div>

      {Object.keys(servers).length === 0 && (
        <p className="empty">No MCP servers configured</p>
      )}
    </div>
  )
}

// Projects View Component
function ProjectsView({ projects, onRefresh, loading }: {
  projects: Project[]
  onRefresh: () => void
  loading: boolean
}) {
  if (loading) return <div className="loading">Loading...</div>

  return (
    <div className="view">
      <h2>Projects</h2>

      <div className="project-list">
        {projects.map((project) => (
          <div key={project.path} className="project-item">
            <h3>{project.name}</h3>
            <p className="path">{project.root}</p>
            <p className="has-config">
              {project.has_config ? '✓ Has config' : 'No config'}
            </p>
          </div>
        ))}
      </div>

      {projects.length === 0 && (
        <p className="empty">No projects found</p>
      )}

      <button onClick={onRefresh} className="btn-primary">
        Refresh
      </button>
    </div>
  )
}

// History View Component
function HistoryView({ backups, onRefresh, loading }: {
  backups: Backup[]
  onRefresh: () => void
  loading: boolean
}) {
  const [restoring, setRestoring] = useState(false)

  const handleRestore = async (backupPath: string) => {
    if (!confirm('Restore this backup?')) return

    setRestoring(true)
    try {
      await invoke('restore_backup', { backupPath })
      alert('Backup restored successfully!')
      onRefresh()
    } catch (err) {
      alert('Failed to restore: ' + err)
    } finally {
      setRestoring(false)
    }
  }

  if (loading) return <div className="loading">Loading...</div>

  return (
    <div className="view">
      <h2>Backup History</h2>

      <div className="backup-list">
        {backups.map((backup) => (
          <div key={backup.path} className="backup-item">
            <div className="backup-info">
              <p className="time">{new Date(backup.created_at).toLocaleString()}</p>
              <p className="size">{backup.size} bytes</p>
            </div>
            <button
              onClick={() => handleRestore(backup.path)}
              disabled={restoring}
              className="btn-small"
            >
              Restore
            </button>
          </div>
        ))}
      </div>

      {backups.length === 0 && (
        <p className="empty">No backups found</p>
      )}

      <button onClick={onRefresh} className="btn-primary">
        Refresh
      </button>
    </div>
  )
}

export default App
