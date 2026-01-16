import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { projectsApi, SentryProject } from '../api/client'
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
  Label,
  EmptyState
} from '../components/ui'
import { Plus, FolderKanban, Edit2, Trash2, RefreshCw, Hash, Info } from 'lucide-react'

type ProjectFormData = {
  project_id: string
  display_name: string
  description?: string
}

import { useLoadingDelay } from '../hooks/useLoadingDelay'

export default function Projects() {
  const [projects, setProjects] = useState<SentryProject[]>([])
  const [loading, setLoading] = useState(true)
  const showLoadingUI = useLoadingDelay(loading)
  const [showForm, setShowForm] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)

  const { register, handleSubmit, reset } = useForm<ProjectFormData>()

  useEffect(() => {
    loadProjects()
  }, [])

  const loadProjects = async () => {
    try {
      setLoading(true)
      const { projects } = await projectsApi.list()
      setProjects(projects)
    } catch (e) {
      console.error('Failed to load projects:', e)
    } finally {
      setLoading(false)
    }
  }

  const onSubmit = async (data: ProjectFormData) => {
    try {
      if (editingId) {
        await projectsApi.update(editingId, data)
      } else {
        await projectsApi.create(data)
      }
      handleCancel()
      loadProjects()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to save project')
    }
  }

  const handleEdit = (project: SentryProject) => {
    setEditingId(project.id)
    reset({
      project_id: project.project_id,
      display_name: project.display_name,
      description: project.description || ''
    })
    setShowForm(true)
  }

  const handleDelete = async (id: number) => {
    if (!confirm('确定删除该项目映射？')) return
    try {
      await projectsApi.delete(id)
      loadProjects()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to delete project')
    }
  }

  const handleCancel = () => {
    setShowForm(false)
    setEditingId(null)
    reset({ project_id: '', display_name: '', description: '' })
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-slate-900 tracking-tight">项目映射</h2>
          <p className="text-slate-500 mt-1">配置 Sentry 项目 ID 与现实显示名称的关联</p>
        </div>
        <Button onClick={() => setShowForm(true)} className="sm:w-auto w-full shadow-sm">
          <Plus className="mr-2 h-4 w-4" />
          添加映射
        </Button>
      </div>

      {/* Projects List */}
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
      ) : projects.length === 0 ? (
        <Card className="border-dashed border-slate-200 shadow-none bg-slate-50/50">
          <CardContent className="pt-12 pb-12">
            <EmptyState
              title="暂无项目映射"
              description="点击上方按钮添加 Sentry 项目 ID 与显示名称的映射"
            />
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {projects.map(project => (
            <Card
              key={project.id}
              className="group overflow-hidden border-slate-200/60 shadow-sm hover:shadow-md transition-all duration-300"
            >
              <CardContent className="p-0">
                <div className="flex flex-col sm:flex-row sm:items-center p-6 gap-6">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="w-12 h-12 rounded-2xl bg-linear-to-br from-emerald-500 to-emerald-600 flex items-center justify-center text-white shrink-0 shadow-sm group-hover:rotate-3 transition-transform duration-300">
                      <FolderKanban className="w-6 h-6" />
                    </div>
                    <div className="space-y-1">
                      <h3 className="font-semibold text-slate-900">{project.display_name}</h3>
                      <div className="flex items-center gap-1.5 text-slate-400">
                        <Hash className="w-3 h-3" />
                        <span className="text-[11px] font-mono tracking-tighter uppercase">
                          {project.project_id}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="flex-1 md:block hidden">
                    {project.description ? (
                      <p className="text-sm text-slate-500 line-clamp-1 italic">
                        {project.description}
                      </p>
                    ) : (
                      <span className="text-xs text-slate-300 italic">暂无项目描述</span>
                    )}
                  </div>

                  <div className="flex items-center gap-1 opacity-100 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleEdit(project)}
                      className="h-9 px-3 text-slate-600 hover:text-primary hover:bg-primary/5"
                    >
                      <Edit2 className="w-4 h-4 mr-1.5" />
                      编辑
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDelete(project.id)}
                      className="h-9 px-3 text-destructive hover:text-destructive hover:bg-destructive/10"
                    >
                      <Trash2 className="w-4 h-4 mr-1.5" />
                      删除
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Form Modal */}
      <Dialog open={showForm} onOpenChange={open => !open && handleCancel()}>
        <DialogContent className="sm:max-w-120 p-0 overflow-hidden border-none shadow-2xl">
          <DialogHeader className="p-6 pb-2">
            <DialogTitle className="text-xl font-bold tracking-tight">
              {editingId ? '编辑项目映射' : '添加项目映射'}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-0">
            <div className="p-6 space-y-6">
              <div className="p-4 bg-primary/5 rounded-2xl border border-primary/10 flex items-start gap-3 mb-2">
                <Info className="w-5 h-5 text-primary shrink-0 mt-0.5" />
                <p className="text-xs text-primary/80 leading-relaxed font-medium">
                  项目 ID 可以在 Sentry 对应项目的 Settings - General
                  页面底部找到（通常是一串数字）。
                </p>
              </div>

              <div className="space-y-5">
                <div className="grid gap-2">
                  <Label htmlFor="project_id" className="text-sm font-semibold text-slate-800">
                    Sentry 项目 ID
                  </Label>
                  <Input
                    id="project_id"
                    {...register('project_id', { required: true })}
                    placeholder="例如: 4505865345040384"
                    required
                    disabled={!!editingId}
                    className="h-10 bg-white"
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="display_name" className="text-sm font-semibold text-slate-800">
                    显示名称
                  </Label>
                  <Input
                    id="display_name"
                    {...register('display_name', { required: true })}
                    placeholder="例如: 前端核心项目"
                    required
                    className="h-10 bg-white"
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="description" className="text-sm font-semibold text-slate-800">
                    描述
                  </Label>
                  <Input
                    id="description"
                    {...register('description')}
                    placeholder="可选简单的描述信息"
                    className="h-10 bg-white"
                  />
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
                保存映射
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
