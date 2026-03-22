"use client"

import * as React from "react"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Label } from "@/components/ui/label"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { DescriptionEditor } from "./description-editor"
import { fetchMyProjects } from "@/lib/my-projects"
import type { Project } from "@/app/[orgSlug]/projects/data/schema"
import { priorities } from "@/app/[orgSlug]/my-issues/data/data"
import {
  HelpCircle,
  Circle,
  Timer,
  CheckCircle,
  XCircle,
  Flame,
  ArrowUp,
  ArrowRight,
  ArrowDown,
  User,
} from "lucide-react"
import { cn } from "@/lib/utils"

const categoryStyle: Record<
  string,
  { icon: React.ElementType; color: string }
> = {
  backlog: { icon: HelpCircle, color: "bg-muted text-muted-foreground" },
  unstarted: {
    icon: Circle,
    color: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
  },
  started: {
    icon: Timer,
    color: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300",
  },
  completed: {
    icon: CheckCircle,
    color: "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
  },
  canceled: {
    icon: XCircle,
    color: "bg-slate-200 text-slate-700 dark:bg-slate-800 dark:text-slate-300",
  },
}

const priorityConfig: Record<
  string,
  { label: string; icon: React.ElementType; color: string }
> = {
  critical: { label: "Critical", icon: Flame, color: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300" },
  high: { label: "High", icon: ArrowUp, color: "bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300" },
  medium: { label: "Medium", icon: ArrowRight, color: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300" },
  low: { label: "Low", icon: ArrowDown, color: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300" },
}

interface ProjectMember {
  user_id: string
  name: string
  email: string
  project_role: string
}

interface WorkflowOption {
  id: string
  category: string
  name: string
  slug: string
  position: number
  is_default: boolean
}

export interface NewIssueModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  orgSlug: string
  initialProjectKey?: string
  initialProjectName?: string
  /** Workflow status UUID (e.g. from project Kanban column). */
  initialWorkflowStatusId?: string
  /** First status in this category when user picks a project (e.g. My Issues board). */
  initialWorkflowCategory?: string
}

export function NewIssueModal({
  open,
  onOpenChange,
  orgSlug,
  initialProjectKey,
  initialProjectName,
  initialWorkflowStatusId: initialWorkflowStatusIdProp,
  initialWorkflowCategory: initialWorkflowCategoryProp,
}: NewIssueModalProps) {
  const [projects, setProjects] = React.useState<Project[]>([])
  const [projectsLoading, setProjectsLoading] = React.useState(false)
  const [members, setMembers] = React.useState<ProjectMember[]>([])
  const [membersLoading, setMembersLoading] = React.useState(false)
  const [workflowOptions, setWorkflowOptions] = React.useState<WorkflowOption[]>([])
  const [workflowLoading, setWorkflowLoading] = React.useState(false)
  const isProjectFixed = Boolean(initialProjectKey)
  const [selectedKey, setSelectedKey] = React.useState<string>(
    initialProjectKey ?? ""
  )
  const [title, setTitle] = React.useState("")
  const [description, setDescription] = React.useState("")
  const [workflowStatusId, setWorkflowStatusId] = React.useState("")
  const [priority, setPriority] = React.useState("medium")
  const [assigneeId, setAssigneeId] = React.useState<string>("")
  const [isSubmitting, setIsSubmitting] = React.useState(false)

  React.useEffect(() => {
    if (!open) return
    if (initialProjectKey) {
      setSelectedKey(initialProjectKey)
      return
    }
    let cancelled = false
    setProjectsLoading(true)
    setSelectedKey("")
    fetchMyProjects({ limit: 100 })
      .then((res) => {
        if (cancelled) return
        const forOrg = res.items.filter((p) => p.org_slug === orgSlug)
        setProjects(forOrg)
        if (forOrg.length > 0) {
          setSelectedKey(forOrg[0].project_key)
        }
      })
      .finally(() => {
        if (!cancelled) setProjectsLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [open, orgSlug, initialProjectKey])

  React.useEffect(() => {
    if (!open || !selectedKey) return
    let cancelled = false
    setWorkflowLoading(true)
    fetch(
      `/api/orgs/${orgSlug}/projects/${selectedKey}/workflow-statuses`,
      { cache: "no-store", credentials: "same-origin" }
    )
      .then((r) => (r.ok ? r.json() : { groups: [] }))
      .then((data: { groups?: { category: string; statuses: WorkflowOption[] }[] }) => {
        if (cancelled) return
        const flat: WorkflowOption[] = []
        for (const g of data.groups ?? []) {
          for (const s of g.statuses) {
            flat.push({ ...s, category: s.category || g.category })
          }
        }
        flat.sort((a, b) => {
          const cat = a.category.localeCompare(b.category)
          if (cat !== 0) return cat
          return a.position - b.position
        })
        setWorkflowOptions(flat)
      })
      .finally(() => {
        if (!cancelled) setWorkflowLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [open, orgSlug, selectedKey])

  React.useEffect(() => {
    if (!open || workflowOptions.length === 0) return
    if (
      initialWorkflowStatusIdProp &&
      workflowOptions.some((o) => o.id === initialWorkflowStatusIdProp)
    ) {
      setWorkflowStatusId(initialWorkflowStatusIdProp)
      return
    }
    if (initialWorkflowCategoryProp) {
      const first = workflowOptions.find(
        (o) => o.category === initialWorkflowCategoryProp
      )
      if (first) {
        setWorkflowStatusId(first.id)
        return
      }
    }
    const d =
      workflowOptions.find((o) => o.is_default) ?? workflowOptions[0]
    if (d) setWorkflowStatusId(d.id)
  }, [
    open,
    initialWorkflowStatusIdProp,
    initialWorkflowCategoryProp,
    workflowOptions,
  ])

  React.useEffect(() => {
    if (!open || !selectedKey) return
    let cancelled = false
    setMembersLoading(true)
    fetch(`/api/orgs/${orgSlug}/projects/${selectedKey}/members`, {
      cache: "no-store",
      credentials: "same-origin",
    })
      .then((res) => (res.ok ? res.json() : { items: [] }))
      .then((data: { items?: ProjectMember[] }) => {
        if (cancelled) return
        setMembers(data.items ?? [])
      })
      .finally(() => {
        if (!cancelled) setMembersLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [open, orgSlug, selectedKey])

  const handleOpenChange = React.useCallback(
    (next: boolean) => {
      if (!next) {
        setTitle("")
        setDescription("")
        setWorkflowStatusId("")
        setPriority("medium")
        setAssigneeId("")
      }
      onOpenChange(next)
    },
    [onOpenChange]
  )

  const handleSubmit = React.useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault()
      if (!selectedKey || !title.trim() || !workflowStatusId) return
      const descriptionToSend =
        description &&
        description !== "<p></p>" &&
        description !== "<p><br></p>" &&
        description.replace(/<[^>]*>/g, "").trim() !== ""
          ? description
          : undefined
      setIsSubmitting(true)
      try {
        const res = await fetch(
          `/api/orgs/${orgSlug}/projects/${selectedKey}/issues`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            credentials: "same-origin",
            body: JSON.stringify({
              title: title.trim(),
              description: descriptionToSend,
              workflow_status_id: workflowStatusId,
              priority,
              assignee_id: assigneeId || null,
            }),
          }
        )
        if (res.ok) {
          handleOpenChange(false)
        }
      } finally {
        setIsSubmitting(false)
      }
    },
    [
      orgSlug,
      selectedKey,
      title,
      description,
      workflowStatusId,
      priority,
      assigneeId,
      handleOpenChange,
    ]
  )

  const displayProjects = isProjectFixed
    ? [{ project_key: initialProjectKey!, name: initialProjectName ?? initialProjectKey! }]
    : projects

  const selectedWf = workflowOptions.find((o) => o.id === workflowStatusId)
  const wfStyle = selectedWf
    ? categoryStyle[selectedWf.category] ?? categoryStyle.unstarted
    : categoryStyle.backlog
  const WfIcon = wfStyle.icon
  const priorityConf = priorityConfig[priority] ?? priorityConfig.medium
  const PriorityIcon = priorityConf.icon
  const assigneeName =
    assigneeId && members.find((m) => m.user_id === assigneeId)?.name

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="flex max-h-[80svh] flex-col overflow-hidden sm:max-w-2xl">
        <DialogHeader className="shrink-0">
          <DialogTitle>New Issue</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden py-2">
          <div className="grid shrink-0 gap-2">
            <Label htmlFor="new-issue-project">Project</Label>
            <Select
              value={selectedKey}
              onValueChange={setSelectedKey}
              disabled={isProjectFixed}
            >
              <SelectTrigger id="new-issue-project" className="w-full">
                <SelectValue
                  placeholder={projectsLoading ? "Loading…" : "Select project"}
                />
              </SelectTrigger>
              <SelectContent>
                {displayProjects.map((p) => (
                  <SelectItem key={p.project_key} value={p.project_key}>
                    {p.name} ({p.project_key})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid shrink-0 gap-2">
            <Label htmlFor="new-issue-title">Title</Label>
            <Input
              id="new-issue-title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Issue title"
              required
            />
          </div>

          <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
            <Label htmlFor="new-issue-description" className="shrink-0">Description</Label>
            <DescriptionEditor
              value={description}
              onChange={setDescription}
              placeholder="Description (optional)"
              className="min-h-0 flex-1 overflow-hidden"
            />
          </div>

          <div className="flex shrink-0 flex-wrap items-center gap-3">
            <Select
              value={workflowStatusId}
              onValueChange={setWorkflowStatusId}
              disabled={workflowLoading || workflowOptions.length === 0}
            >
              <SelectTrigger
                className={cn(
                  "w-fit border-0 bg-transparent p-0 h-auto shadow-none focus:ring-0",
                  wfStyle.color,
                  "rounded-full px-2.5 py-0.5 text-[0.625rem] font-medium"
                )}
              >
                <SelectValue placeholder={workflowLoading ? "Loading…" : "Status"}>
                  {selectedWf ? (
                    <span className="flex items-center gap-1.5">
                      <WfIcon className="size-3" />
                      {selectedWf.name}
                    </span>
                  ) : null}
                </SelectValue>
              </SelectTrigger>
              <SelectContent className="max-h-72">
                {workflowOptions.map((o) => {
                  const c = categoryStyle[o.category] ?? categoryStyle.unstarted
                  const Icon = c.icon
                  return (
                    <SelectItem key={o.id} value={o.id}>
                      <span className="flex items-center gap-2">
                        <Icon className="size-3.5" />
                        <span>{o.name}</span>
                        <span className="text-muted-foreground text-xs">
                          ({o.category})
                        </span>
                      </span>
                    </SelectItem>
                  )
                })}
              </SelectContent>
            </Select>

            <Select value={priority} onValueChange={setPriority}>
              <SelectTrigger
                className={cn(
                  "w-fit border-0 bg-transparent p-0 h-auto shadow-none focus:ring-0",
                  priorityConf.color,
                  "rounded-full px-2.5 py-0.5 text-[0.625rem] font-medium"
                )}
              >
                <SelectValue>
                  <span className="flex items-center gap-1.5">
                    <PriorityIcon className="size-3" />
                    {priorityConf.label}
                  </span>
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {priorities.map((p) => {
                  const c = priorityConfig[p.value] ?? priorityConfig.medium
                  const Icon = c.icon
                  return (
                    <SelectItem key={p.value} value={p.value}>
                      <span className="flex items-center gap-2">
                        <Icon className="size-3.5" />
                        {c.label}
                      </span>
                    </SelectItem>
                  )
                })}
              </SelectContent>
            </Select>

            <Select
                value={assigneeId || "__none__"}
                onValueChange={(v) => setAssigneeId(v === "__none__" ? "" : v)}
                disabled={membersLoading}
              >
                <SelectTrigger
                  className="flex w-fit items-center gap-1.5 rounded-full border border-input bg-muted/50 px-2.5 py-0.5 text-[0.625rem] font-medium h-6 min-w-[7rem] [&_[data-slot=select-value]]:!hidden"
                >
                  <User className="size-3 shrink-0" />
                  <span className="min-w-0 truncate">
                    {membersLoading ? "Loading…" : (assigneeName ?? "Unassigned")}
                  </span>
                  <SelectValue placeholder="Unassigned" />
                </SelectTrigger>
                <SelectContent className="max-w-[14rem]" position="popper" sideOffset={4} align="start">
                  <SelectItem value="__none__">
                    <span className="flex items-center gap-2">
                      <User className="size-3.5" />
                      Unassigned
                    </span>
                  </SelectItem>
                  {members.map((m) => (
                    <SelectItem key={m.user_id} value={m.user_id}>
                      <span className="flex items-center gap-2">
                        <span
                          className="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary text-[10px] font-medium"
                        >
                          {m.name
                            .split(" ")
                            .map((n) => n[0])
                            .join("")
                            .toUpperCase()
                            .slice(0, 2)}
                        </span>
                        {m.name}
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
          </div>

          <div className="flex shrink-0 justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting || !workflowStatusId}>
              {isSubmitting ? "Creating…" : "Create issue"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  )
}
