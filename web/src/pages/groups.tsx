import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { groupsApi, FeishuGroup } from '../api/client'
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
  EmptyState
} from '../components/ui'
import { Plus, Users, Edit2, Trash2, RefreshCw, Info } from 'lucide-react'
import { useLoadingDelay } from '../hooks/useLoadingDelay'

type GroupFormData = Partial<FeishuGroup>

const CARD_COLORS = [
  { value: 'red', label: '红色', bg: 'bg-red-500' },
  { value: 'orange', label: '橙色', bg: 'bg-orange-500' },
  { value: 'yellow', label: '黄色', bg: 'bg-yellow-500' },
  { value: 'green', label: '绿色', bg: 'bg-green-500' },
  { value: 'blue', label: '蓝色', bg: 'bg-blue-500' },
  { value: 'purple', label: '紫色', bg: 'bg-purple-500' }
]

const TEMPLATE_VARS = [
  { var: '{{project}}', desc: '项目名' },
  { var: '{{environment}}', desc: '环境' },
  { var: '{{level}}', desc: '告警级别' },
  { var: '{{title}}', desc: '错误标题' },
  { var: '{{message}}', desc: '错误信息' },
  { var: '{{url}}', desc: 'Sentry 链接' },
  { var: '{{timestamp}}', desc: '时间' },
  { var: '{{triggered_rule}}', desc: '触发规则' },
  { var: '{{platform}}', desc: '平台' },
  { var: '{{culprit}}', desc: '源码位置' },
  { var: '{{user}}', desc: '触发用户' }
]

export default function Groups() {
  const [groups, setGroups] = useState<FeishuGroup[]>([])
  const [loading, setLoading] = useState(true)
  const showLoadingUI = useLoadingDelay(loading)
  const [showForm, setShowForm] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [testing, setTesting] = useState<number | null>(null)

  const { register, handleSubmit, reset, watch, setValue } = useForm<GroupFormData>({
    defaultValues: {
      msg_type: 'interactive',
      card_color: 'red',
      card_show_details_button: true,
      card_title_template: '🚨 Sentry 告警',
      card_config_json: JSON.stringify(DEFAULT_CONFIG)
    }
  })

  // Helper to parse card_config_json
  const getCardConfig = (json: string | null | undefined): CardConfig => {
    if (!json) return DEFAULT_CONFIG
    try {
      return JSON.parse(json)
    } catch {
      return DEFAULT_CONFIG
    }
  }

  const updateConfig = (key: keyof CardConfig, value: any) => {
    const current = getCardConfig(watch('card_config_json'))
    const next = { ...current, [key]: value }
    setValue('card_config_json', JSON.stringify(next))
  }

  useEffect(() => {
    loadGroups()
  }, [])

  const loadGroups = async () => {
    try {
      setLoading(true)
      const { data } = await groupsApi.list()
      setGroups(data)
    } catch (e) {
      console.error('Failed to load groups:', e)
    } finally {
      setLoading(false)
    }
  }

  const onSubmit = async (data: GroupFormData) => {
    try {
      // 强制使用交互式卡片
      const payload: GroupFormData = { ...data, msg_type: 'interactive' }
      if (editingId) {
        await groupsApi.update(editingId, payload)
      } else {
        await groupsApi.create(payload)
      }
      handleCancel()
      loadGroups()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to save group')
    }
  }

  const handleEdit = (group: FeishuGroup) => {
    setEditingId(group.id)
    reset(group)
    setShowForm(true)
  }

  const handleDelete = async (id: number) => {
    if (!confirm('确定删除该飞书群配置？')) return
    try {
      await groupsApi.delete(id)
      loadGroups()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to delete group')
    }
  }

  const handleTest = async (id: number) => {
    setTesting(id)
    try {
      const result = await groupsApi.test(id)
      alert(result.success ? '测试消息发送成功！' : `发送失败: ${result.error}`)
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Test failed')
    } finally {
      setTesting(null)
    }
  }

  const handleCancel = () => {
    setShowForm(false)
    setEditingId(null)
    reset({
      msg_type: 'interactive',
      card_color: 'red',
      card_show_details_button: true,
      card_title_template: '🚨 Sentry 告警',
      card_config_json: JSON.stringify(DEFAULT_CONFIG)
    })
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-slate-900 tracking-tight">飞书群</h2>
          <p className="text-slate-500 mt-1">管理飞书群 Webhook 配置和消息格式</p>
        </div>
        <Button onClick={() => setShowForm(true)} className="sm:w-auto w-full shadow-sm">
          <Plus className="mr-2 h-4 w-4" />
          添加飞书群
        </Button>
      </div>

      {/* Groups List */}
      {showLoadingUI ? (
        <div className="flex items-center justify-center py-24">
          <div className="flex flex-col items-center gap-4">
            <RefreshCw className="h-10 w-10 text-primary animate-spin" />
            <span className="text-slate-500 font-medium">加载中...</span>
          </div>
        </div>
      ) : groups.length === 0 ? (
        <Card className="border-dashed border-slate-200 shadow-none bg-slate-50/50">
          <CardContent className="pt-12 pb-12">
            <EmptyState title="暂无飞书群配置" description="点击上方按钮添加您的第一个飞书群" />
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {groups.map(group => (
            <Card
              key={group.id}
              className="group overflow-hidden border-slate-200/60 shadow-sm hover:shadow-md transition-all duration-300"
            >
              <CardContent className="p-0">
                <div className="flex flex-col sm:flex-row sm:items-center p-6 gap-6">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="w-12 h-12 rounded-2xl bg-linear-to-br from-primary to-primary/80 flex items-center justify-center text-white shrink-0 shadow-sm group-hover:scale-105 transition-transform duration-300">
                      <Users className="w-6 h-6" />
                    </div>
                    <div className="space-y-1">
                      <h3 className="font-semibold text-slate-900">{group.name}</h3>
                      <p className="text-sm text-slate-500 line-clamp-1 max-w-md">
                        {group.description || '暂无描述'}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-4">
                    <Badge
                      variant="secondary"
                      className="px-2.5 py-0.5 font-medium rounded-full bg-blue-50 text-blue-600 border border-blue-100 hover:bg-blue-100 transition-colors"
                    >
                      交互式卡片
                    </Badge>

                    <div className="flex items-center gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleTest(group.id)}
                        disabled={testing === group.id}
                        loading={testing === group.id}
                        className="h-8 px-3 text-slate-600 hover:text-primary hover:bg-primary/5"
                      >
                        测试
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleEdit(group)}
                        className="h-8 px-3 text-slate-600 hover:text-primary hover:bg-primary/5"
                      >
                        <Edit2 className="w-4 h-4 mr-1.5" />
                        编辑
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleDelete(group.id)}
                        className="h-8 px-3 text-destructive hover:text-destructive hover:bg-destructive/10"
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
        <DialogContent className="sm:max-w-160 max-h-[90vh] flex flex-col p-0 overflow-hidden border-none shadow-2xl">
          <DialogHeader className="p-6 pb-2">
            <DialogTitle className="text-xl font-bold tracking-tight">
              {editingId ? '编辑飞书群' : '添加飞书群'}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit(onSubmit)} className="flex-1 flex flex-col min-h-0">
            <div className="flex-1 overflow-y-auto px-6 py-4 space-y-8">
              {/* Basic Info Section */}
              <div className="space-y-4">
                <div className="flex items-center gap-2 mb-2">
                  <div className="w-1 h-4 bg-primary rounded-full" />
                  <h4 className="text-sm font-semibold text-slate-900 uppercase tracking-wider">
                    基础信息
                  </h4>
                </div>
                <div className="grid gap-4">
                  <div className="grid gap-2">
                    <Label htmlFor="name" className="text-sm font-medium">
                      名称
                    </Label>
                    <Input
                      id="name"
                      {...register('name', { required: true })}
                      placeholder="例如: 技术告警群"
                      required
                      className="h-10"
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="webhook_url" className="text-sm font-medium">
                      Webhook URL
                    </Label>
                    <Input
                      id="webhook_url"
                      {...register('webhook_url', { required: true })}
                      placeholder="https://open.feishu.cn/open-apis/bot/v2/hook/xxx"
                      required
                      className="h-10"
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="description" className="text-sm font-medium">
                      描述
                    </Label>
                    <Input
                      id="description"
                      {...register('description')}
                      placeholder="可选描述"
                      className="h-10"
                    />
                  </div>
                </div>
              </div>

              {/* Message Format Section */}
              <div className="space-y-4 pt-4 border-t border-slate-100">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <div className="w-1 h-4 bg-primary rounded-full" />
                    <h4 className="text-sm font-semibold text-slate-900 uppercase tracking-wider">
                      消息格式
                    </h4>
                  </div>
                  <Badge
                    variant="outline"
                    className="text-[10px] font-bold text-primary border-primary/20 bg-primary/5"
                  >
                    仅支持交互式卡片
                  </Badge>
                </div>

                <div className="p-5 bg-slate-50/80 rounded-2xl space-y-6 border border-slate-200/50">
                  <div className="grid gap-2">
                    <Label htmlFor="card_title_template" className="font-semibold text-slate-800">
                      卡片标题模板
                    </Label>
                    <Input
                      id="card_title_template"
                      {...register('card_title_template')}
                      defaultValue="🚨 Sentry 告警"
                      required
                      className="bg-white h-10 rounded-lg shadow-sm"
                    />
                  </div>
                  <TemplateVarsHelp />

                  <div className="space-y-3 pt-2">
                    <Label className="font-semibold text-slate-800">卡片抬头颜色</Label>
                    <div className="flex flex-wrap gap-2.5">
                      {CARD_COLORS.map(color => (
                        <label
                          key={color.value}
                          className={`flex items-center gap-2.5 px-3 py-2 rounded-xl border-2 cursor-pointer transition-all ${
                            watch('card_color') === color.value
                              ? 'border-primary bg-white shadow-md ring-4 ring-primary/5'
                              : 'border-slate-200 bg-white hover:border-slate-300 shadow-sm'
                          }`}
                        >
                          <input
                            type="radio"
                            {...register('card_color')}
                            value={color.value}
                            className="sr-only"
                          />
                          <span className={`w-4 h-4 rounded-full ${color.bg} shadow-xs`} />
                          <span className="text-xs font-bold text-slate-700">{color.label}</span>
                        </label>
                      ))}
                    </div>
                  </div>

                  <div className="flex items-center gap-3 pt-2 group cursor-pointer">
                    <Switch
                      id="show-details"
                      checked={watch('card_show_details_button') === true}
                      onCheckedChange={checked => setValue('card_show_details_button', checked)}
                    />
                    <Label
                      htmlFor="show-details"
                      className="text-sm font-medium text-slate-700 cursor-pointer group-hover:text-primary transition-colors"
                    >
                      显示 "查看详情" 按钮
                    </Label>
                  </div>
                </div>
              </div>

              {/* Card Content Configuration Section */}
              <div className="space-y-4 pt-4 border-t border-slate-100">
                <div className="flex items-center gap-2 mb-2">
                  <div className="w-1 h-4 bg-primary rounded-full" />
                  <h4 className="text-sm font-semibold text-slate-900 uppercase tracking-wider">
                    卡片内容展示配置
                  </h4>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pb-6">
                  {/* Configuration Controls */}
                  <div className="space-y-6">
                    <div className="space-y-3">
                      <Label className="text-xs font-bold text-slate-500 uppercase">主要开关</Label>
                      <div className="grid gap-3">
                        <div className="flex items-center justify-between p-3 bg-slate-50 rounded-xl border border-slate-100">
                          <Label htmlFor="show_exception" className="text-sm font-medium">
                            显示错误详情
                          </Label>
                          <Switch
                            id="show_exception"
                            checked={getCardConfig(watch('card_config_json')).show_exception_detail}
                            onCheckedChange={val => updateConfig('show_exception_detail', val)}
                          />
                        </div>
                        <div className="flex items-center justify-between p-3 bg-slate-50 rounded-xl border border-slate-100">
                          <Label htmlFor="show_tags" className="text-sm font-medium">
                            显示 Tags 标签
                          </Label>
                          <Switch
                            id="show_tags"
                            checked={getCardConfig(watch('card_config_json')).show_tags}
                            onCheckedChange={val => updateConfig('show_tags', val)}
                          />
                        </div>
                      </div>
                    </div>

                    <div className="space-y-3">
                      <Label className="text-xs font-bold text-slate-500 uppercase">字段选择</Label>
                      <div className="bg-slate-50 rounded-2xl border border-slate-100 overflow-hidden">
                        {getCardConfig(watch('card_config_json')).show_fields.map((field, idx) => (
                          <div
                            key={field.key}
                            className={`flex items-center justify-between p-3 hover:bg-white transition-colors ${
                              idx !== 0 ? 'border-t border-slate-100' : ''
                            }`}
                          >
                            <span className="text-sm font-medium text-slate-700">
                              {field.label}
                            </span>
                            <Switch
                              checked={field.enabled}
                              onCheckedChange={checked => {
                                const cfg = getCardConfig(watch('card_config_json'))
                                cfg.show_fields[idx].enabled = checked
                                setValue('card_config_json', JSON.stringify(cfg))
                              }}
                            />
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>

                  {/* Live Preview */}
                  <div className="space-y-3">
                    <Label className="text-xs font-bold text-slate-500 uppercase">
                      实时预览 (模拟效果)
                    </Label>
                    <CardPreview
                      title={watch('card_title_template') || '🚨 Sentry 告警'}
                      color={watch('card_color') || 'red'}
                      showButton={watch('card_show_details_button') === true}
                      config={getCardConfig(watch('card_config_json'))}
                    />
                  </div>
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
              <Button type="submit" className="h-10 px-8 font-semibold shadow-sm">
                保存配置
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}

interface CardConfig {
  show_fields: Array<{ key: string; label: string; enabled: boolean }>
  show_tags: boolean
  show_exception_detail: boolean
}

const DEFAULT_CONFIG: CardConfig = {
  show_fields: [
    { key: 'project', label: '项目', enabled: true },
    { key: 'environment', label: '环境', enabled: true },
    { key: 'level', label: '级别', enabled: true },
    { key: 'timestamp', label: '时间', enabled: true },
    { key: 'platform', label: '平台', enabled: true },
    { key: 'culprit', label: '源码位置', enabled: true },
    { key: 'user', label: '触发用户', enabled: true }
  ],
  show_tags: true,
  show_exception_detail: true
}

function CardPreview({
  title,
  color,
  showButton,
  config
}: {
  title: string
  color: string
  showButton: boolean
  config: CardConfig
}) {
  const colorMap: Record<string, string> = {
    red: 'bg-red-500',
    orange: 'bg-orange-500',
    yellow: 'bg-yellow-500',
    green: 'bg-green-500',
    blue: 'bg-blue-500',
    purple: 'bg-purple-500'
  }

  return (
    <div className="border border-slate-200 rounded-lg overflow-hidden shadow-sm bg-white font-sans text-[12px] max-w-sm">
      {/* Target Logo/Header */}
      <div
        className={`p-3 text-white flex items-center gap-2 ${colorMap[color] || 'bg-slate-500'}`}
      >
        <span className="font-bold truncate">{title}</span>
      </div>

      <div className="p-3 space-y-3">
        {config.show_exception_detail && (
          <>
            <div className="space-y-1">
              <div className="font-bold text-slate-900 leading-tight">
                SyntaxError: Unexpected token 'e', "error code: 500"...
              </div>
              <div className="text-slate-600 line-clamp-2">
                Unexpected token 'e', "error code: 500" is not valid JSON
              </div>
            </div>
            <div className="border-t border-slate-100" />
          </>
        )}

        <div className="grid grid-cols-2 gap-y-2 gap-x-4">
          {config.show_fields
            .filter(f => f.enabled)
            .map(f => (
              <div key={f.key}>
                <div className="text-slate-400 font-bold">{f.label}</div>
                <div className="text-slate-700 truncate capitalize">
                  {f.key === 'project'
                    ? 'maiyuan'
                    : f.key === 'environment'
                    ? 'production'
                    : f.key === 'timestamp'
                    ? '2026-01-16 14:10:26'
                    : 'sample_data'}
                </div>
              </div>
            ))}
        </div>

        {config.show_tags && (
          <div className="pt-1">
            <span className="font-bold text-slate-400 mr-2">Tags:</span>
            <span className="text-slate-600">browser: Safari, os: Mac OS X</span>
          </div>
        )}

        <div className="pt-1 text-slate-400 leading-tight">
          <div>
            <span className="font-bold mr-1">触发规则:</span>
            <span>核心业务监控</span>
          </div>
          <div>
            <span className="font-bold mr-1">Issue ID:</span>
            <span>6978 | </span>
            <span className="font-bold mr-1 px-1">Event ID:</span>
            <span>93b3</span>
          </div>
        </div>

        {showButton && (
          <div className="pt-1">
            <div className="w-full py-1.5 border border-slate-200 rounded text-center text-primary font-bold hover:bg-slate-50 transition-colors cursor-pointer">
              查看详情
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function TemplateVarsHelp() {
  return (
    <div className="p-3 bg-white/60 rounded-xl border border-slate-100 shadow-xs">
      <div className="flex items-center gap-2 mb-2.5 text-slate-600">
        <Info className="w-3.5 h-3.5" />
        <span className="text-[11px] font-bold uppercase tracking-wider">可用动态变量</span>
      </div>
      <div className="flex flex-wrap gap-1.5">
        {TEMPLATE_VARS.map(v => (
          <Badge
            key={v.var}
            variant="outline"
            className="px-2 py-0.5 bg-white text-[10px] font-mono border-slate-200 text-slate-500 hover:text-primary hover:border-primary/30 transition-all cursor-default"
            title={v.desc}
          >
            {v.var}
          </Badge>
        ))}
      </div>
    </div>
  )
}
