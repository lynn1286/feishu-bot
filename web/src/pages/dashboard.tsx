import { useEffect, useState } from 'react'
import { statsApi, Stats } from '../api/client'
import { Button, Card, CardHeader, CardTitle, CardContent, Badge } from '../components/ui'
import { Bell, BarChart3, Users, Route, Zap, Copy, Check, AlertCircle } from 'lucide-react'
import { useLoadingDelay } from '../hooks/useLoadingDelay'

export default function Dashboard() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [loading, setLoading] = useState(true)
  const showLoadingUI = useLoadingDelay(loading)
  const [error, setError] = useState<string | null>(null)
  const [webhookUrl, setWebhookUrl] = useState('')
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    setWebhookUrl(`${window.location.origin}/webhook/sentry`)
    loadStats()
  }, [])

  const loadStats = async () => {
    try {
      setLoading(true)
      const data = await statsApi.get()
      setStats(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load stats')
    } finally {
      setLoading(false)
    }
  }

  const copyWebhookUrl = async () => {
    await navigator.clipboard.writeText(webhookUrl)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const getGreeting = () => {
    const hour = new Date().getHours()
    if (hour < 12) return '早上好'
    if (hour < 18) return '下午好'
    return '晚上好'
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center py-16">
        <div className="w-16 h-16 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
          <AlertCircle className="w-8 h-8 text-destructive" />
        </div>
        <p className="text-slate-600 mb-4">{error}</p>
        <Button onClick={loadStats}>重试</Button>
      </div>
    )
  }

  const statCards = [
    {
      label: '今日告警',
      value: stats?.alerts.today ?? 0,
      icon: <Bell className="w-6 h-6" />,
      color: 'blue',
      gradient: 'from-blue-500 to-blue-600'
    },
    {
      label: '总告警数',
      value: stats?.alerts.total ?? 0,
      icon: <BarChart3 className="w-6 h-6" />,
      color: 'slate',
      gradient: 'from-slate-600 to-slate-700',
      extra: (
        <div className="flex items-center gap-2 mt-2">
          <Badge
            variant="outline"
            className="bg-emerald-50 text-emerald-600 border-emerald-100 font-normal"
          >
            成功: {stats?.alerts.success ?? 0}
          </Badge>
          <Badge variant="outline" className="bg-red-50 text-red-600 border-red-100 font-normal">
            失败: {stats?.alerts.failed ?? 0}
          </Badge>
        </div>
      )
    },
    {
      label: '飞书群',
      value: stats?.groups.total ?? 0,
      icon: <Users className="w-6 h-6" />,
      color: 'purple',
      gradient: 'from-purple-500 to-purple-600'
    },
    {
      label: '路由规则',
      value: stats?.rules.total ?? 0,
      icon: <Route className="w-6 h-6" />,
      color: 'orange',
      gradient: 'from-orange-500 to-orange-600',
      extra: (
        <Badge variant="secondary" className="mt-2 font-normal">
          已启用: {stats?.rules.enabled ?? 0}
        </Badge>
      )
    }
  ]

  const steps = [
    {
      step: 1,
      title: '添加飞书群',
      description: '在飞书群页面添加飞书群的 Webhook URL',
      link: '/groups'
    },
    {
      step: 2,
      title: '配置路由规则',
      description: '在路由规则页面配置告警路由规则',
      link: '/rules'
    },
    {
      step: 3,
      title: '配置 Sentry',
      description: '在 Sentry 中配置 Internal Integration'
    },
    {
      step: 4,
      title: '开始接收告警',
      description: '配置 Sentry 告警规则，触发告警时会自动转发'
    }
  ]

  return (
    <div className="space-y-8">
      {/* Welcome */}
      <div>
        <h2 className="text-2xl font-bold text-slate-900 tracking-tight">{getGreeting()}！</h2>
        <p className="text-slate-500 mt-1">这是您的告警转发系统概览</p>
      </div>

      {/* Security Warning */}
      {stats && !stats.security.sentry_client_secret_set && (
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-amber-600 shrink-0 mt-0.5" />
          <div className="space-y-1">
            <h4 className="font-semibold text-amber-900">未配置 Webhook 签名密钥</h4>
            <p className="text-sm text-amber-700">
              当前未配置{' '}
              <code className="bg-amber-100 px-1 py-0.5 rounded text-amber-800 font-mono text-xs">
                SENTRY_CLIENT_SECRET
              </code>
              ， 系统将跳过 Sentry 来源签名验证。这可能导致伪造的告警请求被处理，存在安全风险。
              建议尽快在环境变量中配置该密钥。
            </p>
          </div>
        </div>
      )}

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {statCards.map((stat, index) => (
          <Card
            key={index}
            className="relative overflow-hidden group hover:shadow-md transition-all duration-300 border-slate-200/60"
          >
            <CardContent className="p-6">
              {/* Gradient decoration */}
              <div
                className={`absolute top-0 right-0 w-32 h-32 bg-linear-to-br ${stat.gradient} opacity-[0.05] rounded-full -translate-y-12 translate-x-12 group-hover:opacity-[0.1] transition-opacity duration-300`}
              />

              <div className="relative">
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-slate-500">{stat.label}</p>
                    {showLoadingUI ? (
                      <div className="h-9 w-16 bg-slate-100 animate-pulse rounded-md mt-1" />
                    ) : (
                      <p className="text-3xl font-bold text-slate-900 mt-1">{stat.value}</p>
                    )}
                    {!loading && stat.extra}
                  </div>
                  <div
                    className={`shrink-0 w-12 h-12 rounded-xl bg-linear-to-br ${stat.gradient} flex items-center justify-center text-white shadow-sm group-hover:shadow-md group-hover:scale-105 transition-all duration-300`}
                  >
                    {stat.icon}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Quick Start Guide */}
      <Card className="border-slate-200/60">
        <CardHeader className="flex-row items-center gap-3 space-y-0">
          <div className="w-10 h-10 rounded-xl bg-linear-to-br from-emerald-500 to-emerald-600 flex items-center justify-center text-white shadow-sm">
            <Zap className="w-5 h-5" />
          </div>
          <div>
            <CardTitle className="text-lg font-semibold text-slate-900">快速开始</CardTitle>
            <p className="text-sm text-slate-500">按照以下步骤配置您的告警转发</p>
          </div>
        </CardHeader>

        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {steps.map(item => (
              <div
                key={item.step}
                className="p-4 rounded-xl border border-slate-200 hover:border-blue-200 hover:bg-blue-50/50 transition-colors group"
              >
                <div className="flex items-center gap-3 mb-2">
                  <span className="w-8 h-8 rounded-full bg-blue-100 text-blue-600 flex items-center justify-center text-sm font-semibold group-hover:bg-blue-600 group-hover:text-white transition-colors">
                    {item.step}
                  </span>
                  <h4 className="font-medium text-slate-900">{item.title}</h4>
                </div>
                <p className="text-sm text-slate-500 ml-11">{item.description}</p>
              </div>
            ))}
          </div>

          {/* Webhook URL */}
          <div className="mt-6 p-4 bg-slate-50 rounded-xl border border-slate-200">
            <p className="text-sm font-medium text-slate-700 mb-2">Sentry Webhook URL</p>
            <div
              onClick={copyWebhookUrl}
              className="flex items-center justify-between gap-3 bg-white px-3 py-2.5 rounded-lg border border-slate-200 cursor-pointer hover:border-primary/30 hover:bg-primary/5 transition-colors group"
            >
              <code className="text-sm text-slate-600 overflow-hidden text-ellipsis flex-1 min-w-0">
                {webhookUrl}
              </code>
              {copied ? (
                <Check className="w-5 h-5 text-emerald-500 shrink-0" />
              ) : (
                <Copy className="w-5 h-5 text-slate-400 group-hover:text-primary shrink-0 transition-colors" />
              )}
            </div>
            {copied && (
              <p className="text-xs text-emerald-600 mt-1.5 font-medium">已复制到剪贴板</p>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
