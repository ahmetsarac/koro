"use client"

import * as React from "react"
import { use } from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"
import {
  Circle,
  Timer,
  CheckCircle,
  HelpCircle,
  Ban,
  User,
  Calendar,
  Clock,
  ArrowLeft,
  Send,
  MessageSquare,
  Pencil,
  Check,
  X,
  Minus,
  ArrowDown,
  ArrowUp,
  Flame,
  ArrowRight,
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { Textarea } from "@/components/ui/textarea"
import { Input } from "@/components/ui/input"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { updateIssueInCaches } from "@/lib/cache/issues-cache"
import { DescriptionEditor } from "@/components/issues/description-editor"

interface Issue {
  issue_id: string
  display_key: string
  title: string
  description: string | null
  status: string
  priority: string
  assignee_id: string | null
  assignee_name: string | null
  project_id: string
  created_at: string
  updated_at: string
}

interface ProjectMember {
  user_id: string
  name: string
  email: string
}

interface Comment {
  comment_id: string
  author_id: string | null
  author_name: string | null
  body: string
  created_at: string
}

const statusConfig: Record<
  string,
  { label: string; icon: React.ElementType; color: string }
> = {
  backlog: { label: "Backlog", icon: HelpCircle, color: "bg-muted text-muted-foreground" },
  todo: { label: "Todo", icon: Circle, color: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300" },
  in_progress: { label: "In Progress", icon: Timer, color: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300" },
  blocked: { label: "Blocked", icon: Ban, color: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300" },
  done: { label: "Done", icon: CheckCircle, color: "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300" },
}

const statusList = ["backlog", "todo", "in_progress", "blocked", "done"] as const

const priorityConfig: Record<
  string,
  { label: string; icon: React.ElementType; color: string }
> = {
  critical: { label: "Critical", icon: Flame, color: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300" },
  high: { label: "High", icon: ArrowUp, color: "bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300" },
  medium: { label: "Medium", icon: ArrowRight, color: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300" },
  low: { label: "Low", icon: ArrowDown, color: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300" },
}

const priorityList = ["critical", "high", "medium", "low"] as const

async function fetchIssue(
  orgSlug: string,
  issueKey: string,
  opts?: { cacheBust?: boolean }
): Promise<Issue> {
  const url = `/api/orgs/${orgSlug}/issues/${issueKey}`
  const finalUrl = opts?.cacheBust ? `${url}?_t=${Date.now()}` : url
  const response = await fetch(finalUrl, {
    cache: "no-store",
    credentials: "same-origin",
    ...(opts?.cacheBust && { headers: { Pragma: "no-cache", "Cache-Control": "no-cache" } }),
  })

  if (!response.ok) {
    throw new Error("Failed to fetch issue")
  }

  return response.json()
}

async function fetchComments(
  orgSlug: string,
  issueKey: string
): Promise<Comment[]> {
  const response = await fetch(
    `/api/orgs/${orgSlug}/issues/${issueKey}/comments`,
    {
      cache: "no-store",
      credentials: "same-origin",
    }
  )

  if (!response.ok) {
    return []
  }

  const data = await response.json()
  return data.items
}

async function createComment(
  orgSlug: string,
  issueKey: string,
  body: string
): Promise<Comment | null> {
  const response = await fetch(
    `/api/orgs/${orgSlug}/issues/${issueKey}/comments`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify({ body }),
    }
  )

  if (!response.ok) {
    return null
  }

  return response.json()
}

async function updateIssue(
  orgSlug: string,
  issueKey: string,
  data: { title?: string; description?: string; priority?: string }
): Promise<boolean> {
  const response = await fetch(`/api/orgs/${orgSlug}/issues/${issueKey}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify(data),
  })

  return response.ok
}

async function assignIssue(
  orgSlug: string,
  issueKey: string,
  userId: string
): Promise<boolean> {
  const response = await fetch(`/api/orgs/${orgSlug}/issues/${issueKey}/assignee`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ user_id: userId }),
  })

  return response.ok
}

async function unassignIssue(
  orgSlug: string,
  issueKey: string
): Promise<boolean> {
  const response = await fetch(`/api/orgs/${orgSlug}/issues/${issueKey}/assignee`, {
    method: "DELETE",
    credentials: "same-origin",
  })

  return response.ok
}

async function fetchProjectMembers(
  orgSlug: string,
  projectId: string
): Promise<ProjectMember[]> {
  const response = await fetch(
    `/api/orgs/${orgSlug}/projects/${projectId}/members`,
    {
      cache: "no-store",
      credentials: "same-origin",
    }
  )

  if (!response.ok) {
    return []
  }

  const data = await response.json()
  return data.items || []
}

async function updateIssueStatus(
  issueId: string,
  status: string
): Promise<boolean> {
  const response = await fetch(`/api/issues/${issueId}/status`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ status }),
  })

  return response.ok
}

export default function IssueDetailPage({
  params,
}: {
  params: Promise<{ orgSlug: string; issueKey: string }>
}) {
  const { orgSlug, issueKey } = use(params)
  const router = useRouter()
  const [issue, setIssue] = React.useState<Issue | null>(null)
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  const [comments, setComments] = React.useState<Comment[]>([])
  const [isLoadingComments, setIsLoadingComments] = React.useState(true)
  const [newComment, setNewComment] = React.useState("")
  const [isSubmitting, setIsSubmitting] = React.useState(false)

  const [isEditingDescription, setIsEditingDescription] = React.useState(false)
  const [editedDescription, setEditedDescription] = React.useState("")
  const [isSavingDescription, setIsSavingDescription] = React.useState(false)

  const [isEditingTitle, setIsEditingTitle] = React.useState(false)
  const [editedTitle, setEditedTitle] = React.useState("")
  const [isSavingTitle, setIsSavingTitle] = React.useState(false)

  const [isUpdatingStatus, setIsUpdatingStatus] = React.useState(false)
  const [isUpdatingPriority, setIsUpdatingPriority] = React.useState(false)
  const [isUpdatingAssignee, setIsUpdatingAssignee] = React.useState(false)
  const [projectMembers, setProjectMembers] = React.useState<ProjectMember[]>([])
  const [isLoadingMembers, setIsLoadingMembers] = React.useState(false)

  React.useEffect(() => {
    async function load() {
      try {
        setIsLoading(true)
        const data = await fetchIssue(orgSlug, issueKey)
        setIssue(data)
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load issue")
      } finally {
        setIsLoading(false)
      }
    }

    load()
  }, [orgSlug, issueKey])

  React.useEffect(() => {
    async function loadComments() {
      try {
        setIsLoadingComments(true)
        const data = await fetchComments(orgSlug, issueKey)
        setComments(data)
      } finally {
        setIsLoadingComments(false)
      }
    }

    loadComments()
  }, [orgSlug, issueKey])

  const handleSubmitComment = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newComment.trim() || isSubmitting) return

    setIsSubmitting(true)
    try {
      const result = await createComment(orgSlug, issueKey, newComment.trim())
      if (result) {
        const updatedComments = await fetchComments(orgSlug, issueKey)
        setComments(updatedComments)
        setNewComment("")
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleStartEditDescription = () => {
    setEditedDescription(issue?.description || "")
    setIsEditingDescription(true)
  }

  const handleCancelEditDescription = () => {
    setIsEditingDescription(false)
    setEditedDescription("")
  }

  const handleSaveDescription = async () => {
    if (isSavingDescription) return

    setIsSavingDescription(true)
    try {
      const success = await updateIssue(orgSlug, issueKey, {
        description: editedDescription,
      })
      if (success && issue) {
        setIssue({ ...issue, description: editedDescription })
        setIsEditingDescription(false)
        updateIssueInCaches(issue.display_key, { description: editedDescription })
      }
    } finally {
      setIsSavingDescription(false)
    }
  }

  const handleStartEditTitle = () => {
    setEditedTitle(issue?.title || "")
    setIsEditingTitle(true)
  }

  const handleCancelEditTitle = () => {
    setIsEditingTitle(false)
    setEditedTitle("")
  }

  const handleSaveTitle = async () => {
    if (isSavingTitle || !editedTitle.trim()) return

    setIsSavingTitle(true)
    try {
      const success = await updateIssue(orgSlug, issueKey, {
        title: editedTitle.trim(),
      })
      if (success && issue) {
        setIssue({ ...issue, title: editedTitle.trim() })
        setIsEditingTitle(false)
        updateIssueInCaches(issue.display_key, { title: editedTitle.trim() })
      }
    } finally {
      setIsSavingTitle(false)
    }
  }

  const handleStatusChange = async (newStatus: string) => {
    if (!issue || isUpdatingStatus || newStatus === issue.status) return

    setIsUpdatingStatus(true)
    try {
      const success = await updateIssueStatus(issue.issue_id, newStatus)
      if (success) {
        setIssue({ ...issue, status: newStatus })
        updateIssueInCaches(issue.display_key, { status: newStatus } as { status?: string })
      }
    } finally {
      setIsUpdatingStatus(false)
    }
  }

  const handlePriorityChange = async (newPriority: string) => {
    if (!issue || isUpdatingPriority || newPriority === issue.priority) return

    setIsUpdatingPriority(true)
    try {
      const success = await updateIssue(orgSlug, issueKey, { priority: newPriority })
      if (success) {
        setIssue({ ...issue, priority: newPriority })
        updateIssueInCaches(issue.display_key, { priority: newPriority })
      }
    } finally {
      setIsUpdatingPriority(false)
    }
  }

  const handleAssigneeChange = async (userId: string | null) => {
    if (!issue || isUpdatingAssignee) return
    if (userId === issue.assignee_id) return

    const assignee_id = userId
    const assignee_name =
      userId === null
        ? null
        : projectMembers.find((m) => m.user_id === userId)?.name ?? null

    setIssue((prev) =>
      prev ? { ...prev, assignee_id, assignee_name } : null
    )
    updateIssueInCaches(issue.display_key, { assignee_id, assignee_name })

    setIsUpdatingAssignee(true)
    try {
      if (userId === null) {
        await unassignIssue(orgSlug, issueKey)
      } else {
        await assignIssue(orgSlug, issueKey, userId)
      }
      try {
        const updated = await fetchIssue(orgSlug, issueKey, { cacheBust: true })
        setIssue(() => updated)
        updateIssueInCaches(updated.display_key, {
          assignee_id: updated.assignee_id,
          assignee_name: updated.assignee_name,
        })
      } catch {
        // Refetch failed; UI already shows optimistic update
      }
    } finally {
      setIsUpdatingAssignee(false)
    }
  }

  const loadProjectMembers = async () => {
    if (!issue || projectMembers.length > 0 || isLoadingMembers) return
    setIsLoadingMembers(true)
    try {
      const projectKey = issue.display_key.split("-")[0]
      const members = await fetchProjectMembers(orgSlug, projectKey)
      setProjectMembers(members)
    } finally {
      setIsLoadingMembers(false)
    }
  }

  if (isLoading) {
    return (
      <div className="flex h-[calc(100svh-4.5rem)] flex-col gap-6">
        <div className="flex items-center gap-4">
          <Skeleton className="h-8 w-8" />
          <Skeleton className="h-6 w-24" />
        </div>
        <Skeleton className="h-10 w-2/3" />
        <Skeleton className="h-6 w-32" />
        <Skeleton className="h-40 w-full" />
      </div>
    )
  }

  if (error || !issue) {
    return (
      <div className="flex h-[calc(100svh-4.5rem)] flex-1 items-center justify-center">
        <div className="text-center">
          <h2 className="text-lg font-semibold text-destructive">Error</h2>
          <p className="text-muted-foreground">{error || "Issue not found"}</p>
          <Button asChild className="mt-4">
            <Link href={`/${orgSlug}/my-issues`}>Back to Issues</Link>
          </Button>
        </div>
      </div>
    )
  }

  const status = statusConfig[issue.status] || {
    label: issue.status,
    icon: Circle,
    color: "bg-muted text-muted-foreground",
  }
  const StatusIcon = status.icon

  const priority = priorityConfig[issue.priority] || {
    label: issue.priority,
    icon: Minus,
    color: "bg-muted text-muted-foreground",
  }
  const PriorityIcon = priority.icon

  const projectKey = issue.display_key.split("-")[0]

  return (
    <div className="flex h-[calc(100svh-4.5rem)] flex-col gap-6">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" className="cursor-pointer" onClick={() => router.back()}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <Link
          href={`/${orgSlug}/projects/${projectKey}`}
          className="font-mono text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          {issue.display_key}
        </Link>
      </div>

      <div className="flex-1 overflow-y-auto">
        {isEditingTitle ? (
          <div className="flex items-center gap-2 mb-4">
            <Input
              value={editedTitle}
              onChange={(e) => setEditedTitle(e.target.value)}
              className="text-2xl font-semibold h-auto py-1 px-2"
              disabled={isSavingTitle}
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault()
                  handleSaveTitle()
                } else if (e.key === "Escape") {
                  handleCancelEditTitle()
                }
              }}
            />
            <Button
              size="icon"
              variant="ghost"
              onClick={handleSaveTitle}
              disabled={isSavingTitle || !editedTitle.trim()}
              className="h-8 w-8 shrink-0"
            >
              <Check className="h-4 w-4" />
            </Button>
            <Button
              size="icon"
              variant="ghost"
              onClick={handleCancelEditTitle}
              disabled={isSavingTitle}
              className="h-8 w-8 shrink-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ) : (
          <h1
            className="text-2xl font-semibold mb-4 cursor-pointer hover:bg-muted/50 rounded px-2 py-1 -mx-2 transition-colors"
            onClick={handleStartEditTitle}
          >
            {issue.title}
          </h1>
        )}

        <div className="flex items-center gap-4 mb-6">
          <DropdownMenu>
            <DropdownMenuTrigger asChild disabled={isUpdatingStatus}>
              <button className="focus:outline-none">
                <Badge className={`${status.color} gap-1.5 cursor-pointer hover:opacity-80 transition-opacity`}>
                  <StatusIcon className="h-3.5 w-3.5" />
                  {status.label}
                </Badge>
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              {statusList.map((statusKey) => {
                const config = statusConfig[statusKey]
                const Icon = config.icon
                return (
                  <DropdownMenuItem
                    key={statusKey}
                    onClick={() => handleStatusChange(statusKey)}
                    className="gap-2"
                  >
                    <Icon className="h-4 w-4" />
                    {config.label}
                    {statusKey === issue.status && (
                      <Check className="h-4 w-4 ml-auto" />
                    )}
                  </DropdownMenuItem>
                )
              })}
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu>
            <DropdownMenuTrigger asChild disabled={isUpdatingPriority}>
              <button className="focus:outline-none">
                <Badge className={`${priority.color} gap-1.5 cursor-pointer hover:opacity-80 transition-opacity`}>
                  <PriorityIcon className="h-3.5 w-3.5" />
                  {priority.label}
                </Badge>
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              {priorityList.map((priorityKey) => {
                const config = priorityConfig[priorityKey]
                const Icon = config.icon
                return (
                  <DropdownMenuItem
                    key={priorityKey}
                    onClick={() => handlePriorityChange(priorityKey)}
                    className="gap-2"
                  >
                    <Icon className="h-4 w-4" />
                    {config.label}
                    {priorityKey === issue.priority && (
                      <Check className="h-4 w-4 ml-auto" />
                    )}
                  </DropdownMenuItem>
                )
              })}
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu
            key={`assignee-${issue.assignee_id ?? "none"}`}
            onOpenChange={(open) => open && loadProjectMembers()}
          >
            <DropdownMenuTrigger asChild disabled={isUpdatingAssignee}>
              <button className="focus:outline-none">
                <Badge variant="outline" className="gap-1.5 cursor-pointer hover:opacity-80 transition-opacity">
                  <User className="h-3.5 w-3.5" />
                  {issue.assignee_name || "Unassigned"}
                </Badge>
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              <DropdownMenuItem
                onClick={() => handleAssigneeChange(null)}
                className="gap-2"
              >
                <User className="h-4 w-4" />
                Unassigned
                {!issue.assignee_id && (
                  <Check className="h-4 w-4 ml-auto" />
                )}
              </DropdownMenuItem>
              {isLoadingMembers ? (
                <DropdownMenuItem disabled className="gap-2">
                  Loading...
                </DropdownMenuItem>
              ) : (
                projectMembers.map((member) => (
                  <DropdownMenuItem
                    key={member.user_id}
                    onClick={() => handleAssigneeChange(member.user_id)}
                    className="gap-2"
                  >
                    <div className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary text-[10px] font-medium">
                      {member.name
                        .split(" ")
                        .map((n) => n[0])
                        .join("")
                        .toUpperCase()
                        .slice(0, 2)}
                    </div>
                    {member.name}
                    {member.user_id === issue.assignee_id && (
                      <Check className="h-4 w-4 ml-auto" />
                    )}
                  </DropdownMenuItem>
                ))
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <div className="flex items-center gap-6 text-sm text-muted-foreground mb-8">
          <div className="flex items-center gap-1.5">
            <Calendar className="h-4 w-4" />
            <span>
              Created{" "}
              {new Date(issue.created_at).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <Clock className="h-4 w-4" />
            <span>
              Updated{" "}
              {new Date(issue.updated_at).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          </div>
        </div>

        <div className="border-t pt-6">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-medium text-muted-foreground">
              Description
            </h2>
            {!isEditingDescription && (
              <Button
                variant="ghost"
                size="sm"
                onClick={handleStartEditDescription}
                className="h-7 px-2 text-muted-foreground hover:text-foreground"
              >
                <Pencil className="h-3.5 w-3.5 mr-1" />
                Edit
              </Button>
            )}
          </div>

          {isEditingDescription ? (
            <div className="space-y-3">
              <DescriptionEditor
                value={editedDescription}
                onChange={setEditedDescription}
                placeholder="Add a description..."
              />
              <div className="flex items-center gap-2">
                <Button
                  size="sm"
                  onClick={handleSaveDescription}
                  disabled={isSavingDescription}
                  className="h-7"
                >
                  <Check className="h-3.5 w-3.5 mr-1" />
                  Save
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleCancelEditDescription}
                  disabled={isSavingDescription}
                  className="h-7"
                >
                  <X className="h-3.5 w-3.5 mr-1" />
                  Cancel
                </Button>
              </div>
            </div>
          ) : issue.description ? (
            <div
              className="prose prose-sm dark:prose-invert max-w-none [&_ul]:my-2 [&_ol]:my-2 [&_li]:my-0.5 [&_p]:my-1 [&_p:first-child]:mt-0 [&_p:last-child]:mb-0"
              dangerouslySetInnerHTML={{ __html: issue.description }}
            />
          ) : (
            <p
              className="text-muted-foreground text-sm italic cursor-pointer hover:text-foreground transition-colors"
              onClick={handleStartEditDescription}
            >
              Click to add a description...
            </p>
          )}
        </div>

        <div className="border-t pt-6 mt-6">
          <h2 className="text-sm font-medium text-muted-foreground mb-4 flex items-center gap-2">
            <MessageSquare className="h-4 w-4" />
            Comments ({comments.length})
          </h2>

          {isLoadingComments ? (
            <div className="space-y-4">
              {Array.from({ length: 2 }).map((_, i) => (
                <div key={i} className="flex gap-3">
                  <Skeleton className="h-8 w-8 rounded-full shrink-0" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-4 w-32" />
                    <Skeleton className="h-12 w-full" />
                  </div>
                </div>
              ))}
            </div>
          ) : comments.length > 0 ? (
            <div className="space-y-4 mb-6">
              {comments.map((comment) => (
                <div key={comment.comment_id} className="flex gap-3">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary text-xs font-medium">
                    {comment.author_name
                      ? comment.author_name
                          .split(" ")
                          .map((n) => n[0])
                          .join("")
                          .toUpperCase()
                          .slice(0, 2)
                      : "?"}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-sm font-medium">
                        {comment.author_name || "Unknown"}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {new Date(comment.created_at).toLocaleDateString(
                          "en-US",
                          {
                            month: "short",
                            day: "numeric",
                            hour: "2-digit",
                            minute: "2-digit",
                          }
                        )}
                      </span>
                    </div>
                    <p className="text-sm whitespace-pre-wrap break-words">
                      {comment.body}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground text-sm italic mb-6">
              No comments yet.
            </p>
          )}

          <form onSubmit={handleSubmitComment} className="relative">
            <Textarea
              placeholder="Leave a comment..."
              value={newComment}
              onChange={(e) => setNewComment(e.target.value)}
              className="min-h-[100px] resize-none text-sm pr-12 pb-10"
              disabled={isSubmitting}
            />
            <Button
              type="submit"
              size="icon"
              variant="ghost"
              disabled={!newComment.trim() || isSubmitting}
              className="absolute bottom-2 right-2 h-7 w-7 rounded-md border border-dashed border-muted-foreground/50 text-muted-foreground hover:text-foreground hover:border-foreground disabled:opacity-30"
            >
              <Send className="h-4 w-4" />
            </Button>
          </form>
        </div>
      </div>
    </div>
  )
}
