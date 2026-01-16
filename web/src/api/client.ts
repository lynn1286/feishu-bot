const API_BASE = '/api'

function getAuthHeader(): Record<string, string> {
  const token = localStorage.getItem('admin_token')
  return token ? { 'X-Admin-Token': token } : {}
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      ...getAuthHeader(),
      ...options?.headers
    },
    ...options
  })

  if (response.status === 401) {
    localStorage.removeItem('admin_token')
    window.location.reload()
    throw new Error('Unauthorized')
  }

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }))
    throw new Error(error.error || `HTTP ${response.status}`)
  }

  if (response.status === 204) {
    return {} as T
  }

  return response.json()
}

// Types
export interface FeishuGroup {
  id: number
  name: string
  webhook_url: string
  description: string | null
  msg_type: 'text' | 'post' | 'interactive'
  text_template: string | null
  post_title_template: string | null
  card_title_template: string | null
  card_color: string | null
  card_show_details_button: boolean | null
  card_config_json: string | null
  created_at: string
  updated_at: string
}

export interface RoutingRule {
  id: number
  name: string
  project_match: string | null
  environment_match: string | null
  level_match: string | null
  group_id: number
  priority: number
  enabled: number
  created_at: string
  updated_at: string
  group_name?: string
}

export interface RoutingRuleInput {
  name?: string
  project_match?: string | null
  environment_match?: string | null
  level_match?: string | null
  group_id?: number
  priority?: number
  enabled?: boolean
}

export interface AlertHistory {
  id: number
  sentry_event_id: string | null
  project: string | null
  environment: string | null
  title: string | null
  message: string | null
  level: string | null
  matched_rule_id: number | null
  target_group_id: number | null
  target_group_name: string | null
  status: string
  error_message: string | null
  raw_payload: string | null
  created_at: string
}

export interface Stats {
  alerts: {
    total: number
    success: number
    failed: number
    today: number
  }
  groups: {
    total: number
  }
  rules: {
    total: number
    enabled: number
  }
  security: {
    sentry_client_secret_set: boolean
  }
}

export interface SentryProject {
  id: number
  project_id: string
  display_name: string
  description: string | null
  created_at: string
  updated_at: string
}

// Groups API
export const groupsApi = {
  list: () => request<{ data: FeishuGroup[] }>('/groups'),
  get: (id: number) => request<{ data: FeishuGroup }>(`/groups/${id}`),
  create: (data: Partial<FeishuGroup>) =>
    request<{ data: FeishuGroup }>('/groups', {
      method: 'POST',
      body: JSON.stringify(data)
    }),
  update: (id: number, data: Partial<FeishuGroup>) =>
    request<{ data: FeishuGroup }>(`/groups/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data)
    }),
  delete: (id: number) => request<void>(`/groups/${id}`, { method: 'DELETE' }),
  test: (id: number) =>
    request<{ success: boolean; message?: string; error?: string }>(`/groups/${id}/test`, {
      method: 'POST'
    })
}

// Rules API
export const rulesApi = {
  list: () => request<{ data: RoutingRule[] }>('/rules'),
  get: (id: number) => request<{ data: RoutingRule }>(`/rules/${id}`),
  create: (data: RoutingRuleInput) =>
    request<{ data: RoutingRule }>('/rules', {
      method: 'POST',
      body: JSON.stringify(data)
    }),
  update: (id: number, data: RoutingRuleInput) =>
    request<{ data: RoutingRule }>(`/rules/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data)
    }),
  delete: (id: number) => request<void>(`/rules/${id}`, { method: 'DELETE' })
}

// Alerts API
export const alertsApi = {
  list: (params?: { page?: number; page_size?: number; status?: string; project?: string }) => {
    const searchParams = new URLSearchParams()
    if (params?.page) searchParams.set('page', params.page.toString())
    if (params?.page_size) searchParams.set('page_size', params.page_size.toString())
    if (params?.status) searchParams.set('status', params.status)
    if (params?.project) searchParams.set('project', params.project)
    const query = searchParams.toString()
    return request<{
      data: AlertHistory[]
      total: number
      page: number
      page_size: number
      total_pages: number
    }>(`/alerts${query ? `?${query}` : ''}`)
  },
  get: (id: number) => request<{ data: AlertHistory }>(`/alerts/${id}`),
  retry: (id: number) =>
    request<{ success: boolean; message?: string; error?: string }>(`/alerts/${id}/retry`, {
      method: 'POST'
    })
}

// Stats API
export const statsApi = {
  get: () => request<Stats>('/stats')
}

// Projects API
export const projectsApi = {
  list: () => request<{ projects: SentryProject[] }>('/projects'),
  get: (id: number) => request<SentryProject>(`/projects/${id}`),
  create: (data: Partial<SentryProject>) =>
    request<SentryProject>('/projects', {
      method: 'POST',
      body: JSON.stringify(data)
    }),
  update: (id: number, data: Partial<SentryProject>) =>
    request<SentryProject>(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data)
    }),
  delete: (id: number) => request<{ success: boolean }>(`/projects/${id}`, { method: 'DELETE' })
}
