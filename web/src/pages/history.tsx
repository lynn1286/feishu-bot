import { useEffect, useState } from 'react'
import { alertsApi, AlertHistory, projectsApi, SentryProject } from '../api/client'
import {
  Button,
  Card,
  CardContent,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Badge,
  EmptyState,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '../components/ui'
import {
  RefreshCw,
  Search,
  History as HistoryIcon,
  ChevronRight,
  ChevronLeft,
  Info,
  AlertCircle,
  CheckCircle2,
  XCircle,
  HelpCircle,
  Clock,
  RotateCcw,
  Terminal,
  Copy,
  Check
} from 'lucide-react'

import { useLoadingDelay } from '../hooks/useLoadingDelay'

export default function History() {
  const [alerts, setAlerts] = useState<AlertHistory[]>([])
  const [projects, setProjects] = useState<SentryProject[]>([])
  const [loading, setLoading] = useState(true)
  const showLoadingUI = useLoadingDelay(loading)
  const [page, setPage] = useState(1)
  const [totalPages, setTotalPages] = useState(1)
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [projectFilter, setProjectFilter] = useState<string>('')
  const [selectedAlert, setSelectedAlert] = useState<AlertHistory | null>(null)
  const [retrying, setRetrying] = useState<number | null>(null)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    loadData()
  }, [page, statusFilter, projectFilter])

  const loadData = async () => {
    try {
      setLoading(true)
      const [alertsResult, projectsResult] = await Promise.all([
        alertsApi.list({
          page,
          page_size: 20,
          status: statusFilter || undefined,
          project: projectFilter || undefined
        }),
        projectsApi.list()
      ])
      setAlerts(alertsResult.data)
      setTotalPages(alertsResult.total_pages)
      setProjects(projectsResult.projects)
    } catch (e) {
      console.error('Failed to load data:', e)
    } finally {
      setLoading(false)
    }
  }

  const handleRetry = async (id: number) => {
    setRetrying(id)
    try {
      const result = await alertsApi.retry(id)
      alert(result.success ? '重试成功！' : `重试失败: ${result.error}`)
      loadData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Retry failed')
    } finally {
      setRetrying(null)
    }
  }

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const parseDate = (dateStr: string) => {
    // Check if dateStr contains timezone info (Z or +HH:MM/-HH:MM)
    const hasTimezone = dateStr.endsWith('Z') || /[+-]\d{2}:?\d{2}$/.test(dateStr)
    if (hasTimezone) return new Date(dateStr)

    // Treat as UTC if missing timezone
    const normalized = dateStr.replace(' ', 'T')
    return new Date(`${normalized}Z`)
  }

  const formatDate = (dateStr: string) => parseDate(dateStr).toLocaleString('zh-CN')

  const formatRelativeTime = (dateStr: string) => {
    const date = parseDate(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)
    if (minutes < 1) return '刚刚'
    if (minutes < 60) return `${minutes} 分钟前`
    if (hours < 24) return `${hours} 小时前`
    if (days < 7) return `${days} 天前`
    return formatDate(dateStr)
  }

  const getStatusConfig = (status: string) => {
    const configs: Record<
      string,
      { variant: 'default' | 'destructive' | 'outline' | 'secondary'; label: string; icon: any }
    > = {
      success: { variant: 'default', label: '成功', icon: CheckCircle2 },
      failed: { variant: 'destructive', label: '失败', icon: XCircle },
      no_match: { variant: 'outline', label: '未匹配', icon: HelpCircle },
      verification_failed: { variant: 'destructive', label: '验签失败', icon: XCircle }
    }
    return configs[status] || { variant: 'secondary' as const, label: status, icon: Info }
  }

  const getProjectName = (projectId: string | null) => {
    if (!projectId) return 'Unknown'
    const project = projects.find(p => p.project_id === projectId)
    return project ? project.display_name : projectId
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-slate-900 tracking-tight">告警历史</h2>
          <p className="text-slate-500 mt-1">查看所有告警的处理历史记录和原始数据</p>
        </div>
        <Button onClick={() => loadData()} className="sm:w-auto w-full shadow-sm group">
          <RefreshCw
            className={`mr-2 h-4 w-4 ${
              loading
                ? 'animate-spin'
                : 'group-hover:rotate-180 transition-transform duration-500 text-white/80'
            }`}
          />
          刷新数据
        </Button>
      </div>

      {/* Filters Section */}
      <Card className="border-slate-200/60 shadow-sm bg-slate-50/30">
        <CardContent className="p-4">
          <div className="flex flex-col sm:flex-row items-end gap-4">
            <div className="w-full sm:w-48 space-y-1.5">
              <Label
                htmlFor="status-filter"
                className="text-xs font-bold text-slate-500 uppercase tracking-wider ml-1"
              >
                处理状态
              </Label>
              <Select
                value={statusFilter || 'all'}
                onValueChange={(value: string) => {
                  setStatusFilter(value === 'all' ? '' : value)
                  setPage(1)
                }}
              >
                <SelectTrigger id="status-filter" className="w-full bg-white h-10 shadow-xs">
                  <SelectValue placeholder="显示全部" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">显示全部</SelectItem>
                  <SelectItem value="success">成功 (Success)</SelectItem>
                  <SelectItem value="failed">失败 (Failed)</SelectItem>
                  <SelectItem value="no_match">未匹配 (No Match)</SelectItem>
                  <SelectItem value="verification_failed">验签失败 (Auth Failed)</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="w-full sm:w-72 space-y-1.5">
              <Label
                htmlFor="project-search"
                className="text-xs font-bold text-slate-500 uppercase tracking-wider ml-1"
              >
                项目搜索
              </Label>
              <div className="relative">
                <Input
                  id="project-search"
                  value={projectFilter}
                  onChange={e => {
                    setProjectFilter(e.target.value)
                    setPage(1)
                  }}
                  placeholder="输入项目名关键词..."
                  className="pl-9 bg-white h-10 shadow-xs"
                />
                <Search className="absolute left-3 top-3 w-4 h-4 text-slate-400" />
              </div>
            </div>

            {(statusFilter || projectFilter) && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setStatusFilter('')
                  setProjectFilter('')
                  setPage(1)
                }}
                className="text-slate-500 h-10 px-4 hover:text-primary transition-colors"
              >
                清除所有筛选
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Alerts Table */}
      {loading ? (
        <div className="flex items-center justify-center py-24">
          <div className="flex flex-col items-center gap-4">
            {showLoadingUI && (
              <>
                <RefreshCw className="h-10 w-10 text-primary animate-spin" />
                <span className="text-slate-500 font-medium">加载中...</span>
              </>
            )}
          </div>
        </div>
      ) : alerts.length === 0 ? (
        <Card className="border-dashed border-slate-200 shadow-none bg-slate-50/50">
          <CardContent className="py-20">
            <EmptyState
              title="暂无告警记录"
              description={
                statusFilter || projectFilter
                  ? '当前筛选条件下暂无任何告警记录'
                  : '当收到 Sentry 告警请求时，处理历史记录将显示在这里'
              }
            />
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          <Card className="overflow-hidden border-slate-200/60 shadow-sm shadow-slate-200/50">
            <div className="overflow-x-auto">
              <table className="w-full min-w-200 border-collapse">
                <thead>
                  <tr className="bg-slate-50/80 border-b border-slate-100">
                    <th className="px-6 py-4 text-left text-[11px] font-bold text-slate-400 uppercase tracking-widest">
                      时间节点
                    </th>
                    <th className="px-6 py-4 text-left text-[11px] font-bold text-slate-400 uppercase tracking-widest">
                      状态详情
                    </th>
                    <th className="px-6 py-4 text-left text-[11px] font-bold text-slate-400 uppercase tracking-widest">
                      所属项目
                    </th>
                    <th className="px-6 py-4 text-left text-[11px] font-bold text-slate-400 uppercase tracking-widest">
                      告警摘要
                    </th>
                    <th className="px-6 py-4 text-left text-[11px] font-bold text-slate-400 uppercase tracking-widest">
                      投递至
                    </th>
                    <th className="px-6 py-4 text-right text-[11px] font-bold text-slate-400 uppercase tracking-widest">
                      操作
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {alerts.map(alert => {
                    const status = getStatusConfig(alert.status)
                    const StatusIcon = status.icon
                    return (
                      <tr
                        key={alert.id}
                        className="hover:bg-slate-50/30 transition-colors duration-200 group"
                      >
                        <td className="px-6 py-4 whitespace-nowrap">
                          <div className="flex items-center gap-2">
                            <Clock className="w-3.5 h-3.5 text-slate-300" />
                            <span
                              className="text-sm text-slate-600 font-medium"
                              title={formatDate(alert.created_at)}
                            >
                              {formatRelativeTime(alert.created_at)}
                            </span>
                          </div>
                        </td>
                        <td className="px-6 py-4">
                          <Badge
                            variant={status.variant}
                            className="gap-1.5 px-2.5 py-0.5 rounded-full font-bold"
                          >
                            <StatusIcon className="w-3 h-3" />
                            {status.label}
                          </Badge>
                        </td>
                        <td className="px-6 py-4">
                          <div className="text-sm font-bold text-slate-900">
                            {getProjectName(alert.project)}
                          </div>
                        </td>
                        <td className="px-6 py-4 max-w-xs">
                          <div
                            className="text-sm text-slate-600 truncate font-medium group-hover:text-primary transition-colors"
                            title={alert.title || ''}
                          >
                            {alert.title || '-'}
                          </div>
                        </td>
                        <td className="px-6 py-4">
                          <div className="text-sm text-slate-500 font-medium">
                            {alert.target_group_name || '未匹配路由'}
                          </div>
                        </td>
                        <td className="px-6 py-4 text-right">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setSelectedAlert(alert)}
                            className="h-8 w-8 p-0 rounded-lg hover:bg-primary/5 hover:text-primary transition-all shadow-none"
                          >
                            <ChevronRight className="w-4 h-4" />
                          </Button>
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>
          </Card>

          {/* Pagination Section */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between px-2">
              <div className="text-xs font-bold text-slate-400 tracking-wider">
                共计 {totalPages} 页数据
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.max(1, p - 1))}
                  disabled={page === 1}
                  className="h-9 px-3 bg-white shadow-xs border-slate-200"
                >
                  <ChevronLeft className="mr-1.5 h-4 w-4" />
                  上一页
                </Button>
                <div className="flex items-center justify-center min-w-10 h-9 rounded-lg bg-slate-100 text-slate-700 text-sm font-bold border border-slate-200 shadow-inner">
                  {page}
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                  disabled={page === totalPages}
                  className="h-9 px-3 bg-white shadow-xs border-slate-200"
                >
                  下一页
                  <ChevronRight className="ml-1.5 h-4 w-4" />
                </Button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Alert Detail Modal */}
      <Dialog open={!!selectedAlert} onOpenChange={open => !open && setSelectedAlert(null)}>
        <DialogContent className="sm:max-w-180 max-h-[92vh] flex flex-col p-0 overflow-hidden border-none shadow-2xl">
          <DialogHeader className="p-6 pb-2 border-b border-slate-50 bg-slate-50/20">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-xl bg-primary/10 text-primary">
                <HistoryIcon className="w-5 h-5" />
              </div>
              <DialogTitle className="text-xl font-bold tracking-tight">告警处理详情</DialogTitle>
            </div>
          </DialogHeader>

          {selectedAlert && (
            <div className="flex-1 flex flex-col min-h-0">
              <div className="flex-1 overflow-y-auto p-6 space-y-8">
                {/* Status and Info Grid */}
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 p-5 bg-slate-50/80 rounded-2xl border border-slate-200/60 shadow-inner-sm">
                  <div className="space-y-1">
                    <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider block">
                      当前状态
                    </span>
                    <div>
                      {(() => {
                        const s = getStatusConfig(selectedAlert.status)
                        const SIcon = s.icon
                        return (
                          <Badge
                            variant={s.variant}
                            className="gap-1.5 px-2.5 py-0.5 rounded-full font-bold"
                          >
                            <SIcon className="w-3.5 h-3.5" />
                            {s.label}
                          </Badge>
                        )
                      })()}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider block">
                      创建时间
                    </span>
                    <div className="text-sm font-bold text-slate-700">
                      {formatDate(selectedAlert.created_at)}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider block">
                      源项目
                    </span>
                    <div className="text-sm font-bold text-slate-700">
                      {getProjectName(selectedAlert.project)}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider block">
                      目标飞书群
                    </span>
                    <div className="text-sm font-bold text-primary">
                      {selectedAlert.target_group_name || '未匹配'}
                    </div>
                  </div>
                </div>

                {/* Content Section */}
                <div className="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
                  <div className="grid gap-2">
                    <div className="flex items-center gap-2 text-slate-900 mb-1">
                      <div className="w-1 h-3.5 bg-primary rounded-full" />
                      <Label className="text-sm font-bold uppercase tracking-tight">告警标题</Label>
                    </div>
                    <div className="p-4 bg-white rounded-xl border border-slate-100 shadow-sm text-sm font-medium leading-relaxed text-slate-800">
                      {selectedAlert.title || '无标题'}
                    </div>
                  </div>

                  <div className="grid gap-2">
                    <div className="flex items-center gap-2 text-slate-900 mb-1">
                      <div className="w-1 h-3.5 bg-primary rounded-full" />
                      <Label className="text-sm font-bold uppercase tracking-tight">
                        核心内容摘要
                      </Label>
                    </div>
                    <div className="p-4 bg-white rounded-xl border border-slate-100 shadow-sm text-sm font-medium leading-relaxed text-slate-800 whitespace-pre-wrap">
                      {selectedAlert.message || '没有预览内容'}
                    </div>
                  </div>

                  {selectedAlert.error_message && (
                    <div className="grid gap-2">
                      <div className="flex items-center gap-2 text-destructive mb-1">
                        <AlertCircle className="w-4 h-4" />
                        <Label className="text-sm font-bold uppercase tracking-tight">
                          处理异常信息
                        </Label>
                      </div>
                      <div className="p-4 bg-destructive/5 border border-destructive/10 rounded-xl text-sm font-bold text-destructive leading-relaxed">
                        {selectedAlert.error_message}
                      </div>
                    </div>
                  )}

                  <div className="grid gap-2">
                    <div className="flex items-center justify-between mb-1">
                      <div className="flex items-center gap-2 text-slate-900">
                        <Terminal className="w-4 h-4" />
                        <Label className="text-sm font-bold uppercase tracking-tight">
                          原始 JSON 数据负载
                        </Label>
                      </div>
                      <Badge
                        variant="outline"
                        className="text-[9px] uppercase font-bold tracking-widest text-slate-400"
                      >
                        Read Only
                      </Badge>
                    </div>
                    <div className="relative group/raw">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="absolute top-2 right-2 h-6 w-6 p-0 opacity-50 hover:opacity-100 transition-opacity bg-slate-800/50 hover:bg-slate-700 text-slate-300 z-10"
                        onClick={() => {
                          try {
                            const jsonStr = JSON.stringify(
                              JSON.parse(selectedAlert.raw_payload || '{}'),
                              null,
                              2
                            )
                            handleCopy(jsonStr)
                          } catch (e) {
                            handleCopy(selectedAlert.raw_payload || '')
                          }
                        }}
                      >
                        {copied ? (
                          <Check className="w-3.5 h-3.5 text-emerald-500" />
                        ) : (
                          <Copy className="w-3.5 h-3.5" />
                        )}
                      </Button>
                      <pre className="p-5 bg-slate-950 text-emerald-400 rounded-2xl text-[12px] font-mono whitespace-pre-wrap break-all border border-slate-800 shadow-2xl custom-scrollbar">
                        {(() => {
                          try {
                            return JSON.stringify(
                              JSON.parse(selectedAlert.raw_payload || '{}'),
                              null,
                              2
                            )
                          } catch (e) {
                            return selectedAlert.raw_payload
                          }
                        })()}
                      </pre>
                    </div>
                  </div>
                </div>
              </div>

              <DialogFooter className="p-6 pt-4 border-t border-slate-50 bg-slate-50/50">
                <div className="flex w-full justify-between items-center">
                  <div className="flex items-center gap-4">
                    {selectedAlert.status === 'failed' && selectedAlert.target_group_id && (
                      <Button
                        onClick={() => handleRetry(selectedAlert.id)}
                        disabled={retrying === selectedAlert.id}
                        loading={retrying === selectedAlert.id}
                        className="h-10 px-6 font-bold"
                      >
                        <RotateCcw className="w-4 h-4 mr-2" />
                        立即重试投递
                      </Button>
                    )}
                  </div>
                  <Button
                    variant="outline"
                    onClick={() => setSelectedAlert(null)}
                    className="h-10 px-8 font-semibold shadow-xs"
                  >
                    完成并关闭
                  </Button>
                </div>
              </DialogFooter>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
