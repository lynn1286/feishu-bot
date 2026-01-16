import { useEffect, useState } from 'react'
import { useForm, Controller } from 'react-hook-form'
import {
  rulesApi,
  groupsApi,
  projectsApi,
  RoutingRule,
  RoutingRuleInput,
  FeishuGroup,
  SentryProject
} from '../api/client'
import {
  Button,
  Card,
  CardContent,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Input,
  Badge,
  Switch,
  Label,
  EmptyState,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '../components/ui'
import {
  Plus,
  Settings2,
  Trash2,
  RefreshCw,
  AlertTriangle,
  ArrowRight,
  ShieldCheck,
  ShieldAlert,
  Layers,
  Globe,
  Terminal
} from 'lucide-react'

interface RuleFormData {
  name: string
  project_match?: string | null
  environment_match?: string | null
  level_match?: string | null
  group_id: string
  priority: number
  enabled: boolean
}

import { useLoadingDelay } from '../hooks/useLoadingDelay'

export default function Rules() {
  const [rules, setRules] = useState<RoutingRule[]>([])
  const [groups, setGroups] = useState<FeishuGroup[]>([])
  const [projects, setProjects] = useState<SentryProject[]>([])
  const [loading, setLoading] = useState(true)
  const showLoadingUI = useLoadingDelay(loading)
  const [showForm, setShowForm] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)

  const { register, handleSubmit, reset, control } = useForm<RuleFormData>({
    defaultValues: { priority: 0, enabled: true }
  })

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      setLoading(true)
      const [rulesRes, groupsRes, projectsRes] = await Promise.all([
        rulesApi.list(),
        groupsApi.list(),
        projectsApi.list()
      ])
      setRules(rulesRes.data)
      setGroups(groupsRes.data)
      setProjects(projectsRes.projects)
    } catch (e) {
      console.error('Failed to load data:', e)
    } finally {
      setLoading(false)
    }
  }

  const onSubmit = async (data: RuleFormData) => {
    try {
      const payload: RoutingRuleInput = {
        name: data.name,
        project_match: data.project_match || null,
        environment_match: data.environment_match || null,
        level_match: data.level_match || null,
        group_id: Number(data.group_id),
        priority: Number(data.priority) || 0,
        enabled: data.enabled === true
      }
      if (editingId) {
        await rulesApi.update(editingId, payload)
      } else {
        await rulesApi.create(payload)
      }
      handleCancel()
      loadData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to save rule')
    }
  }

  const handleEdit = (rule: RoutingRule) => {
    setEditingId(rule.id)
    reset({
      name: rule.name,
      project_match: rule.project_match,
      environment_match: rule.environment_match,
      level_match: rule.level_match,
      group_id: String(rule.group_id),
      priority: rule.priority,
      enabled: rule.enabled === 1
    })
    setShowForm(true)
  }

  const handleDelete = async (id: number) => {
    if (!confirm('确定删除该路由规则？')) return
    try {
      await rulesApi.delete(id)
      loadData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to delete rule')
    }
  }

  const handleToggle = async (rule: RoutingRule) => {
    try {
      await rulesApi.update(rule.id, { enabled: rule.enabled !== 1 })
      loadData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to toggle rule')
    }
  }

  const handleCancel = () => {
    setShowForm(false)
    setEditingId(null)
    reset({ priority: 0, enabled: true })
  }

  const getProjectDisplayName = (projectId: string | null | undefined) => {
    if (!projectId) return null
    const project = projects.find(p => p.project_id === projectId)
    return project ? project.display_name : projectId
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-slate-900 tracking-tight">路由规则</h2>
          <p className="text-slate-500 mt-1">配置告警路由规则，将不同告警转发到对应的飞书群</p>
        </div>
        <Button
          onClick={() => setShowForm(true)}
          disabled={groups.length === 0}
          className="sm:w-auto w-full shadow-sm"
        >
          <Plus className="mr-2 h-4 w-4" />
          添加规则
        </Button>
      </div>

      {groups.length === 0 && !loading && (
        <Card className="border-amber-200 bg-amber-50/50 shadow-none">
          <CardContent className="p-4 flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
            <div>
              <p className="font-semibold text-amber-900">请先添加飞书群配置</p>
              <p className="text-sm text-amber-700 mt-0.5">
                在创建路由规则之前，您需要先添加至少一个飞书群以供选择。
              </p>
            </div>
          </CardContent>
        </Card>
      )}

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
      ) : rules.length === 0 ? (
        <Card className="border-dashed border-slate-200 shadow-none bg-slate-50/50">
          <CardContent className="pt-12 pb-12">
            <EmptyState title="暂无路由规则" description="点击上方按钮添加您的第一条路由规则" />
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {rules
            .sort((a, b) => b.priority - a.priority)
            .map(rule => (
              <Card
                key={rule.id}
                className={`group overflow-hidden border-slate-200/60 shadow-sm transition-all duration-300 ${
                  rule.enabled !== 1 ? 'opacity-60 bg-slate-50/50' : 'hover:shadow-md'
                }`}
              >
                <CardContent className="p-0">
                  <div className="flex flex-col sm:flex-row sm:items-center p-6 gap-6">
                    <div className="flex items-center gap-4">
                      <div className="flex flex-col items-center justify-center p-2 rounded-xl bg-slate-100/80 border border-slate-200/60 min-w-14 shadow-xs">
                        <span className="text-[10px] uppercase font-bold text-slate-400 tracking-tighter">
                          优先级
                        </span>
                        <span className="text-lg font-black text-slate-700 leading-none mt-1">
                          {rule.priority}
                        </span>
                      </div>
                      <Switch
                        checked={rule.enabled === 1}
                        onCheckedChange={() => handleToggle(rule)}
                      />
                    </div>

                    <div className="flex-1 min-w-0 space-y-2">
                      <div className="flex items-center gap-3">
                        <h3 className="font-bold text-slate-900 text-lg tracking-tight truncate">
                          {rule.name}
                        </h3>
                        <Badge
                          variant={rule.enabled === 1 ? 'outline' : 'secondary'}
                          className={
                            rule.enabled === 1 ? 'border-primary/20 text-primary bg-primary/5' : ''
                          }
                        >
                          {rule.enabled === 1 ? '已启用' : '已禁用'}
                        </Badge>
                      </div>

                      <div className="flex flex-wrap items-center gap-2">
                        {rule.project_match ? (
                          <Badge
                            variant="outline"
                            className="gap-1 px-2 font-medium bg-white text-slate-600 border-slate-200"
                          >
                            <Layers className="w-3 h-3 text-slate-400" />
                            项目: {getProjectDisplayName(rule.project_match)}
                          </Badge>
                        ) : null}
                        {rule.environment_match ? (
                          <Badge
                            variant="outline"
                            className="gap-1 px-2 font-medium bg-white text-slate-600 border-slate-200"
                          >
                            <Globe className="w-3 h-3 text-slate-400" />
                            环境: {rule.environment_match}
                          </Badge>
                        ) : null}
                        {rule.level_match ? (
                          <Badge
                            variant="outline"
                            className="gap-1 px-2 font-medium bg-white text-slate-600 border-slate-200"
                          >
                            <ShieldAlert className="w-3 h-3 text-slate-400" />
                            级别: {rule.level_match}
                          </Badge>
                        ) : null}
                        {!rule.project_match && !rule.environment_match && !rule.level_match && (
                          <span className="text-xs text-slate-400 italic">
                            匹配所有告警数据流入
                          </span>
                        )}
                      </div>
                    </div>

                    <div className="flex items-center gap-6">
                      <div className="flex flex-col items-end">
                        <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider">
                          转发至飞书群
                        </span>
                        <div className="flex items-center gap-1.5 mt-0.5">
                          <ArrowRight className="w-3.5 h-3.5 text-primary" />
                          <span className="font-bold text-slate-700">
                            {rule.group_name || '未指定'}
                          </span>
                        </div>
                      </div>

                      <div className="flex items-center gap-1 opacity-100 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity duration-200">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleEdit(rule)}
                          className="h-9 px-3 text-slate-600 hover:text-primary hover:bg-primary/5"
                        >
                          <Settings2 className="w-4 h-4 mr-1.5" />
                          配置
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDelete(rule.id)}
                          className="h-9 px-3 text-destructive hover:text-destructive hover:bg-destructive/10"
                        >
                          <Trash2 className="w-4 h-4 mr-1.5" />
                          删除
                        </Button>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
        </div>
      )}

      {/* Form Modal */}
      <Dialog open={showForm} onOpenChange={open => !open && handleCancel()}>
        <DialogContent className="sm:max-w-140 max-h-[90vh] flex flex-col p-0 overflow-hidden border-none shadow-2xl">
          <DialogHeader className="p-6 pb-2">
            <DialogTitle className="text-xl font-bold tracking-tight">
              {editingId ? '编辑规则' : '添加规则'}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit(onSubmit)} className="flex-1 flex flex-col min-h-0">
            <div className="flex-1 overflow-y-auto px-6 py-4 space-y-6">
              <div className="grid gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="name" className="text-sm font-semibold text-slate-800">
                    规则名称
                  </Label>
                  <Input
                    id="name"
                    {...register('name', { required: true })}
                    placeholder="例如: 生产环境致命错误告警"
                    required
                    className="h-10"
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="group_id" className="text-sm font-semibold text-slate-800">
                    目标飞书群
                  </Label>
                  <Controller
                    name="group_id"
                    control={control}
                    rules={{ required: true }}
                    render={({ field }) => (
                      <Select
                        onValueChange={(value: string) => field.onChange(Number(value))}
                        value={field.value?.toString()}
                      >
                        <SelectTrigger className="h-10 w-full bg-white shadow-sm">
                          <SelectValue placeholder="点击选择目标群..." />
                        </SelectTrigger>
                        <SelectContent>
                          {groups.map(group => (
                            <SelectItem key={group.id} value={group.id.toString()}>
                              {group.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    )}
                  />
                </div>
              </div>

              <div className="p-5 bg-slate-50/80 rounded-2xl border border-slate-200/50 space-y-4">
                <div className="flex items-center gap-2 mb-1">
                  <ShieldCheck className="w-4 h-4 text-primary" />
                  <h4 className="text-sm font-bold text-slate-800 uppercase tracking-tight">
                    匹配条件
                  </h4>
                </div>
                <p className="text-[11px] text-slate-500 leading-relaxed font-medium">
                  项目名必须匹配 Sentry 项目标识。环境与级别支持通配符{' '}
                  <code className="bg-white px-1 border rounded text-primary">*</code>{' '}
                  匹配任意字符，多个值用英文逗号分隔。
                </p>

                <div className="space-y-4 pt-1">
                  <div className="grid gap-1.5">
                    <Label htmlFor="project_match" className="text-xs font-bold text-slate-600">
                      关联项目
                    </Label>
                    <Controller
                      name="project_match"
                      control={control}
                      render={({ field }) => (
                        <Select
                          onValueChange={(value: string) =>
                            field.onChange(value === 'all' ? '' : value)
                          }
                          value={field.value || 'all'}
                        >
                          <SelectTrigger className="h-9 w-full bg-white shadow-xs">
                            <SelectValue placeholder="匹配所有可用的 Sentry 项目" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="all">匹配所有可用的 Sentry 项目</SelectItem>
                            {projects.map(project => (
                              <SelectItem key={project.project_id} value={project.project_id}>
                                {project.display_name}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      )}
                    />
                  </div>

                  <div className="grid gap-1.5">
                    <Label htmlFor="environment_match" className="text-xs font-bold text-slate-600">
                      环境 (Environment)
                    </Label>
                    <Input
                      id="environment_match"
                      {...register('environment_match')}
                      placeholder="例如: production, staging* (留空匹配所有)"
                      className="h-9 bg-white"
                    />
                  </div>

                  <div className="grid gap-1.5">
                    <Label htmlFor="level_match" className="text-xs font-bold text-slate-600">
                      告警级别 (Level)
                    </Label>
                    <Input
                      id="level_match"
                      {...register('level_match')}
                      placeholder="例如: error, fatal (留空匹配所有)"
                      className="h-9 bg-white"
                    />
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-6 items-end pt-2 border-t border-slate-100">
                <div className="grid gap-2">
                  <Label htmlFor="priority" className="text-sm font-semibold text-slate-800">
                    优先级
                  </Label>
                  <div className="relative">
                    <Input
                      id="priority"
                      type="number"
                      {...register('priority')}
                      placeholder="0"
                      className="h-10 pl-10"
                    />
                    <div className="absolute inset-y-0 left-0 flex items-center pl-3 text-slate-400">
                      <Terminal className="w-4 h-4" />
                    </div>
                  </div>
                  <p className="text-[10px] text-slate-500 font-medium">
                    数值越大，规则被触发的优先级越高
                  </p>
                </div>

                <div className="flex items-center gap-3 h-10 pb-0.5 group cursor-pointer">
                  <Controller
                    name="enabled"
                    control={control}
                    render={({ field }) => (
                      <Switch id="enabled" checked={field.value} onCheckedChange={field.onChange} />
                    )}
                  />
                  <Label
                    htmlFor="enabled"
                    className="text-sm font-bold text-slate-700 cursor-pointer group-hover:text-primary transition-colors"
                  >
                    激活规则
                  </Label>
                </div>
              </div>
            </div>

            <DialogFooter className="p-6 pt-4 border-t border-slate-50 bg-slate-50/30">
              <Button
                type="button"
                variant="outline"
                onClick={handleCancel}
                className="h-10 px-6 font-medium"
              >
                取消
              </Button>
              <Button type="submit" className="h-10 px-8 font-bold shadow-sm">
                应用规则
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
