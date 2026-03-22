"use client"

import * as React from "react"
import { use } from "react"
import Link from "next/link"
import {
  FolderKanban,
  Users,
  FileText,
  Plus,
  LayoutGrid,
  List,
  Circle,
  Timer,
  CheckCircle,
  HelpCircle,
  Ban,
  RotateCw,
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { NewIssueModal } from "@/components/issues/new-issue-modal"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { projectRoles } from "@/app/[orgSlug]/projects/data/data"
import { ProjectKanbanBoard } from "./components/project-kanban-board"
import { Skeleton } from "@/components/ui/skeleton"
import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

interface Project {
  id: string
  project_key: string
  name: string
  description: string | null
  org_id: string
  org_name: string
  org_slug: string
  issue_count: number
  member_count: number
  my_role: string
  created_at: string
}

async function fetchProject(
  orgSlug: string,
  projectKey: string
): Promise<Project> {
  const response = await fetch(`/api/orgs/${orgSlug}/projects/${projectKey}`, {
    cache: "no-store",
    credentials: "same-origin",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch project")
  }

  return response.json()
}

export default function ProjectDetailPage({
  params,
}: {
  params: Promise<{ orgSlug: string; projectKey: string }>
}) {
  const { orgSlug, projectKey } = use(params)
  const [project, setProject] = React.useState<Project | null>(null)
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)
  const [newIssueOpen, setNewIssueOpen] = React.useState(false)
  const [newIssueInitialStatus, setNewIssueInitialStatus] = React.useState<string | undefined>(undefined)

  React.useEffect(() => {
    async function load() {
      try {
        setIsLoading(true)
        const data = await fetchProject(orgSlug, projectKey)
        setProject(data)
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load project")
      } finally {
        setIsLoading(false)
      }
    }

    load()
  }, [orgSlug, projectKey])

  if (isLoading) {
    return (
      <div className="flex flex-col gap-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-96" />
        <Skeleton className="h-10 w-full mt-4" />
      </div>
    )
  }

  if (error || !project) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="text-center">
          <h2 className="text-lg font-semibold text-destructive">Error</h2>
          <p className="text-muted-foreground">{error || "Project not found"}</p>
          <Button asChild className="mt-4">
            <Link href={`/${orgSlug}/projects`}>Back to Projects</Link>
          </Button>
        </div>
      </div>
    )
  }

  const role = projectRoles.find((r) => r.value === project.my_role)

  return (
    <div className="flex h-[calc(100svh-4.5rem)] flex-col gap-4">
      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-3">
            <FolderKanban className="h-6 w-6 text-muted-foreground" />
            <h1 className="text-2xl font-semibold">{project.name}</h1>
          </div>
          {project.description && (
            <p className="text-muted-foreground mt-1 max-w-2xl">
              {project.description}
            </p>
          )}
        </div>

        <Button data-icon="inline-start" onClick={() => setNewIssueOpen(true)}>
          <Plus className="h-4 w-4" />
          New Issue
        </Button>
        <NewIssueModal
          open={newIssueOpen}
          onOpenChange={(open) => {
            setNewIssueOpen(open)
            if (!open) setNewIssueInitialStatus(undefined)
          }}
          orgSlug={orgSlug}
          initialProjectKey={projectKey}
          initialProjectName={project.name}
          initialStatus={newIssueInitialStatus}
        />
      </div>

      <div className="flex items-center gap-6 text-sm">
        <div className="flex items-center gap-1.5 text-muted-foreground">
          <FileText className="h-4 w-4" />
          <span>
            {project.issue_count} issue{project.issue_count !== 1 && "s"}
          </span>
        </div>
        <div className="flex items-center gap-1.5 text-muted-foreground">
          <Users className="h-4 w-4" />
          <span>
            {project.member_count} member{project.member_count !== 1 && "s"}
          </span>
        </div>
        {role && (
          <div className="flex items-center gap-1.5">
            {role.icon && <role.icon className="h-4 w-4 text-muted-foreground" />}
            <span className="text-muted-foreground">{role.label}</span>
          </div>
        )}
      </div>

      <Tabs defaultValue="issues" className="flex min-h-0 flex-1 flex-col mt-2">
        <TabsList>
          <TabsTrigger value="issues" className="gap-2">
            <List className="h-4 w-4" />
            Issues
          </TabsTrigger>
          <TabsTrigger value="board" className="gap-2">
            <LayoutGrid className="h-4 w-4" />
            Board
          </TabsTrigger>
          <TabsTrigger value="members" className="gap-2">
            <Users className="h-4 w-4" />
            Members
          </TabsTrigger>
        </TabsList>

        <TabsContent value="issues" className="mt-4 flex min-h-0 flex-1 flex-col">
          <ProjectIssuesTab orgSlug={orgSlug} projectKey={projectKey} />
        </TabsContent>

        <TabsContent value="board" className="mt-4 flex min-h-0 flex-1 flex-col">
          <ProjectKanbanBoard
            orgSlug={orgSlug}
            projectKey={projectKey}
            onAddIssue={(columnId: string) => {
              setNewIssueInitialStatus(columnId)
              setNewIssueOpen(true)
            }}
          />
        </TabsContent>

        <TabsContent value="members" className="mt-4 flex min-h-0 flex-1 flex-col">
          <ProjectMembersTab orgSlug={orgSlug} projectKey={projectKey} />
        </TabsContent>
      </Tabs>
    </div>
  )
}

interface IssueListItem {
  issue_id: string
  display_key: string
  title: string
  status: string
}

interface IssuesResponse {
  items: IssueListItem[]
  total: number
  limit: number
  offset: number
  has_more: boolean
}

const statusConfig: Record<
  string,
  { label: string; icon: React.ElementType }
> = {
  backlog: { label: "Backlog", icon: HelpCircle },
  todo: { label: "Todo", icon: Circle },
  in_progress: { label: "In Progress", icon: Timer },
  blocked: { label: "Blocked", icon: Ban },
  done: { label: "Done", icon: CheckCircle },
}

const ROW_HEIGHT = 49
const OVERSCAN = 2
const PAGE_SIZE = 50
const LOAD_MORE_THRESHOLD = 300

const COL_WIDTHS = {
  key: 100,
  title: "auto",
  status: 140,
}

import {
  projectIssuesCache,
  type ProjectIssuesScrollState,
} from "@/lib/cache/issues-cache"

interface IssuesScrollState {
  scrollTop: number
  items: IssueListItem[]
  total: number
  hasMore: boolean
}

function getIssuesCacheKey(orgSlug: string, projectKey: string): string {
  return `${orgSlug}-${projectKey}`
}

function saveIssuesScrollState(
  orgSlug: string,
  projectKey: string,
  state: IssuesScrollState
): void {
  projectIssuesCache.set(
    getIssuesCacheKey(orgSlug, projectKey),
    state as ProjectIssuesScrollState
  )
}

function loadIssuesScrollState(
  orgSlug: string,
  projectKey: string
): IssuesScrollState | null {
  return projectIssuesCache.get(
    getIssuesCacheKey(orgSlug, projectKey)
  ) as IssuesScrollState | null ?? null
}

function ProjectIssuesTab({
  orgSlug,
  projectKey,
}: {
  orgSlug: string
  projectKey: string
}) {
  const [items, setItems] = React.useState<IssueListItem[]>([])
  const [total, setTotal] = React.useState(0)
  const [hasMore, setHasMore] = React.useState(true)
  const [isInitialLoading, setIsInitialLoading] = React.useState(true)
  const [isFetchingMore, setIsFetchingMore] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const [scrollTop, setScrollTop] = React.useState(0)
  const [containerHeight, setContainerHeight] = React.useState(0)
  const [listBodyOverflows, setListBodyOverflows] = React.useState(false)
  const scrollContainerRef = React.useRef<HTMLDivElement>(null)
  const hasRestoredFromCacheRef = React.useRef(false)
  const pendingScrollTopRef = React.useRef<number | null>(null)

  const fetchIssues = React.useCallback(
    async (offset: number) => {
      const response = await fetch(
        `/api/orgs/${orgSlug}/projects/${projectKey}/issues?limit=${PAGE_SIZE}&offset=${offset}`,
        {
          cache: "no-store",
          credentials: "same-origin",
        }
      )

      if (!response.ok) {
        throw new Error("Failed to fetch issues")
      }

      return response.json() as Promise<IssuesResponse>
    },
    [orgSlug, projectKey]
  )

  // İlk yükleme: cache'den restore et veya API'den yükle
  React.useEffect(() => {
    // Cache'den restore edildiyse Strict Mode ikinci çalışmasında hiçbir şey yapma
    if (hasRestoredFromCacheRef.current) {
      return
    }

    let cancelled = false

    async function init() {
      // Cache'den restore dene
      const savedState = loadIssuesScrollState(orgSlug, projectKey)
      if (savedState && savedState.items.length > 0) {
        hasRestoredFromCacheRef.current = true
        setItems(savedState.items)
        setTotal(savedState.total)
        setHasMore(savedState.hasMore)
        setIsInitialLoading(false)
        pendingScrollTopRef.current = savedState.scrollTop
        return
      }

      // Cache yoksa API'den yükle
      try {
        setIsInitialLoading(true)
        setError(null)
        setItems([])

        const res = await fetchIssues(0)

        if (cancelled) return

        setItems(res.items)
        setTotal(res.total)
        setHasMore(res.has_more)
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : "Failed to load issues.")
        }
      } finally {
        if (!cancelled) setIsInitialLoading(false)
      }
    }

    init()
    return () => {
      cancelled = true
    }
  }, [fetchIssues, orgSlug, projectKey])

  // Scroll pozisyonunu restore et
  React.useEffect(() => {
    if (
      pendingScrollTopRef.current !== null &&
      !isInitialLoading &&
      items.length > 0
    ) {
      const scrollTopValue = pendingScrollTopRef.current
      pendingScrollTopRef.current = null
      requestAnimationFrame(() => {
        scrollContainerRef.current?.scrollTo({ top: scrollTopValue })
        setScrollTop(scrollTopValue)
      })
    }
  }, [isInitialLoading, items.length])

  // State'i sessionStorage'a kaydet
  React.useEffect(() => {
    if (items.length > 0 && !isInitialLoading) {
      saveIssuesScrollState(orgSlug, projectKey, {
        scrollTop,
        items,
        total,
        hasMore,
      })
    }
  }, [scrollTop, items, total, hasMore, orgSlug, projectKey, isInitialLoading])

  const loadMore = React.useCallback(async () => {
    if (isFetchingMore || !hasMore) return

    try {
      setIsFetchingMore(true)

      const res = await fetchIssues(items.length)

      setItems((prev) => [...prev, ...res.items])
      setTotal(res.total)
      setHasMore(res.has_more)
    } catch {
      // silent
    } finally {
      setIsFetchingMore(false)
    }
  }, [fetchIssues, hasMore, isFetchingMore, items.length])

  const visibleCount =
    containerHeight > 0 ? Math.ceil(containerHeight / ROW_HEIGHT) : 12

  const startIndex = Math.max(
    0,
    Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN
  )
  const endIndex = Math.min(
    items.length,
    startIndex + visibleCount + OVERSCAN * 2
  )

  const visibleItems = items.slice(startIndex, endIndex)
  const topSpacerHeight = startIndex * ROW_HEIGHT
  const bottomSpacerHeight = Math.max(0, (items.length - endIndex) * ROW_HEIGHT)

  const updateIssuesListScrollMetrics = React.useCallback(() => {
    const container = scrollContainerRef.current
    if (!container) return
    setContainerHeight(container.clientHeight)
    setListBodyOverflows(container.scrollHeight > container.clientHeight + 1)
  }, [])

  React.useLayoutEffect(() => {
    const container = scrollContainerRef.current
    if (!container) return

    updateIssuesListScrollMetrics()
    const observer = new ResizeObserver(updateIssuesListScrollMetrics)
    observer.observe(container)
    return () => observer.disconnect()
  }, [
    updateIssuesListScrollMetrics,
    items.length,
    topSpacerHeight,
    bottomSpacerHeight,
  ])

  const handleScroll = React.useCallback(
    (event: React.UIEvent<HTMLDivElement>) => {
      const el = event.currentTarget
      setScrollTop(el.scrollTop)

      const distanceToBottom =
        el.scrollHeight - el.scrollTop - el.clientHeight

      if (
        distanceToBottom < LOAD_MORE_THRESHOLD &&
        hasMore &&
        !isFetchingMore
      ) {
        void loadMore()
      }
    },
    [hasMore, isFetchingMore, loadMore]
  )

  const colGroup = (
    <colgroup>
      <col style={{ width: COL_WIDTHS.key }} />
      <col />
      <col style={{ width: COL_WIDTHS.status }} />
    </colgroup>
  )

  const ISSUES_HEADER_HEIGHT = 41
  const issuesListContentHeight =
    ISSUES_HEADER_HEIGHT + items.length * ROW_HEIGHT

  if (isInitialLoading) {
    return (
      <div className="space-y-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-12 w-full" />
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center text-destructive py-12 border rounded-md">
        <p>{error}</p>
      </div>
    )
  }

  if (items.length === 0) {
    return (
      <div className="text-center text-muted-foreground py-12 border rounded-md">
        <p>No issues in this project yet.</p>
        <p className="text-sm mt-2">Create your first issue to get started.</p>
      </div>
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <div className="relative flex max-h-full min-h-0 flex-1 flex-col overflow-hidden">
        <div
          className="relative flex flex-col overflow-hidden rounded-md border"
          style={{ height: `min(${issuesListContentHeight}px, 100%)` }}
        >
          <table className="w-full text-xs" style={{ tableLayout: "fixed" }}>
            {colGroup}
            <TableHeader>
              <TableRow className="border-none">
                <TableHead className="bg-background shadow-[inset_0_-1px_0_0_var(--border)]">
                  Key
                </TableHead>
                <TableHead className="bg-background shadow-[inset_0_-1px_0_0_var(--border)]">
                  Title
                </TableHead>
                <TableHead className="bg-background shadow-[inset_0_-1px_0_0_var(--border)]">
                  Status
                </TableHead>
              </TableRow>
            </TableHeader>
          </table>

          <div
            ref={scrollContainerRef}
            className={
              listBodyOverflows
                ? "relative min-h-0 flex-1 overflow-y-auto"
                : "relative min-h-0 flex-1 overflow-y-hidden"
            }
            onScroll={handleScroll}
          >
            <table className="w-full text-xs" style={{ tableLayout: "fixed" }}>
              {colGroup}
              <TableBody>
                {topSpacerHeight > 0 && (
                  <TableRow aria-hidden="true" className="hover:bg-transparent">
                    <TableCell
                      colSpan={3}
                      className="p-0"
                      style={{ height: topSpacerHeight }}
                    />
                  </TableRow>
                )}

                {visibleItems.map((issue) => {
                  const status = statusConfig[issue.status] || {
                    label: issue.status,
                    icon: Circle,
                  }
                  const StatusIcon = status.icon

                  return (
                    <TableRow key={issue.issue_id} style={{ height: ROW_HEIGHT }}>
                      <TableCell className="font-mono text-xs">
                        <Link
                          href={`/${orgSlug}/issue/${issue.display_key}`}
                          className="hover:underline"
                        >
                          {issue.display_key}
                        </Link>
                      </TableCell>
                      <TableCell>
                        <Link
                          href={`/${orgSlug}/issue/${issue.display_key}`}
                          className="hover:underline truncate block"
                        >
                          {issue.title}
                        </Link>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <StatusIcon className="h-4 w-4 text-muted-foreground" />
                          <span>{status.label}</span>
                        </div>
                      </TableCell>
                    </TableRow>
                  )
                })}

                {bottomSpacerHeight > 0 && (
                  <TableRow aria-hidden="true" className="hover:bg-transparent">
                    <TableCell
                      colSpan={3}
                      className="p-0"
                      style={{ height: bottomSpacerHeight }}
                    />
                  </TableRow>
                )}
              </TableBody>
            </table>
          </div>
        </div>
      </div>

      <div className="relative flex items-center justify-center px-2 py-1 text-sm text-muted-foreground">
        <span>{total > 0 ? `${items.length} / ${total}` : "—"}</span>

        {isFetchingMore && (
          <span className="absolute right-2 flex items-center text-xs">
            <RotateCw className="w-3 h-3 mr-1.5 animate-spin" />
            Loading…
          </span>
        )}
      </div>
    </div>
  )
}

interface ProjectMember {
  user_id: string
  name: string
  email: string
  project_role: string
}

interface MembersResponse {
  items: ProjectMember[]
}

const MEMBER_ROW_HEIGHT = 56
const MEMBER_OVERSCAN = 2

const roleLabels: Record<string, string> = {
  owner: "Owner",
  admin: "Admin",
  member: "Member",
  viewer: "Viewer",
}

function ProjectMembersTab({
  orgSlug,
  projectKey,
}: {
  orgSlug: string
  projectKey: string
}) {
  const [members, setMembers] = React.useState<ProjectMember[]>([])
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  const [scrollTop, setScrollTop] = React.useState(0)
  const [containerHeight, setContainerHeight] = React.useState(0)
  const [membersBodyOverflows, setMembersBodyOverflows] = React.useState(false)
  const scrollContainerRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    async function loadMembers() {
      try {
        setIsLoading(true)
        setError(null)

        const response = await fetch(
          `/api/orgs/${orgSlug}/projects/${projectKey}/members`,
          {
            cache: "no-store",
            credentials: "same-origin",
          }
        )

        if (!response.ok) {
          throw new Error("Failed to fetch members")
        }

        const data: MembersResponse = await response.json()
        setMembers(data.items || [])
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load members")
      } finally {
        setIsLoading(false)
      }
    }

    loadMembers()
  }, [orgSlug, projectKey])

  const visibleCount =
    containerHeight > 0 ? Math.ceil(containerHeight / MEMBER_ROW_HEIGHT) : 12

  const startIndex = Math.max(
    0,
    Math.floor(scrollTop / MEMBER_ROW_HEIGHT) - MEMBER_OVERSCAN
  )
  const endIndex = Math.min(
    members.length,
    startIndex + visibleCount + MEMBER_OVERSCAN * 2
  )

  const visibleMembers = members.slice(startIndex, endIndex)
  const topSpacerHeight = startIndex * MEMBER_ROW_HEIGHT
  const bottomSpacerHeight = Math.max(
    0,
    (members.length - endIndex) * MEMBER_ROW_HEIGHT
  )

  const updateMembersScrollMetrics = React.useCallback(() => {
    const container = scrollContainerRef.current
    if (!container) return
    setContainerHeight(container.clientHeight)
    setMembersBodyOverflows(container.scrollHeight > container.clientHeight + 1)
  }, [])

  React.useLayoutEffect(() => {
    const container = scrollContainerRef.current
    if (!container) return

    updateMembersScrollMetrics()
    const observer = new ResizeObserver(updateMembersScrollMetrics)
    observer.observe(container)
    return () => observer.disconnect()
  }, [
    updateMembersScrollMetrics,
    members.length,
    topSpacerHeight,
    bottomSpacerHeight,
  ])

  const handleScroll = React.useCallback(
    (event: React.UIEvent<HTMLDivElement>) => {
      setScrollTop(event.currentTarget.scrollTop)
    },
    []
  )

  if (isLoading) {
    return (
      <div className="space-y-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-14 w-full" />
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center text-destructive py-12 border rounded-md">
        <p>{error}</p>
      </div>
    )
  }

  if (members.length === 0) {
    return (
      <div className="text-center text-muted-foreground py-12 border rounded-md">
        <p>No members in this project yet.</p>
      </div>
    )
  }

  const HEADER_HEIGHT = 44
  const contentHeight = HEADER_HEIGHT + members.length * MEMBER_ROW_HEIGHT

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div
        className="relative flex flex-col rounded-md border overflow-hidden"
        style={{ height: `min(${contentHeight}px, 100%)` }}
      >
        <div className="flex items-center gap-4 p-3 border-b bg-muted/30 text-sm font-medium text-muted-foreground">
          <div className="flex-1">Member</div>
          <div className="w-32">Role</div>
        </div>

        <div
          ref={scrollContainerRef}
          className={
            membersBodyOverflows
              ? "relative min-h-0 flex-1 overflow-y-auto"
              : "relative min-h-0 flex-1 overflow-y-hidden"
          }
          onScroll={handleScroll}
        >
          {topSpacerHeight > 0 && (
            <div style={{ height: topSpacerHeight }} aria-hidden="true" />
          )}

          {visibleMembers.map((member) => (
            <div
              key={member.user_id}
              className="flex items-center gap-4 px-3 border-b last:border-b-0 hover:bg-muted/50 transition-colors"
              style={{ height: MEMBER_ROW_HEIGHT }}
            >
              <div className="flex flex-1 items-center gap-3 min-w-0">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary text-sm font-medium">
                  {member.name
                    .split(" ")
                    .map((n) => n[0])
                    .join("")
                    .slice(0, 2)
                    .toUpperCase()}
                </div>
                <div className="min-w-0">
                  <p className="font-medium truncate">{member.name}</p>
                  <p className="text-sm text-muted-foreground truncate">
                    {member.email}
                  </p>
                </div>
              </div>
              <div className="w-32">
                <span className="inline-flex items-center rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium">
                  {roleLabels[member.project_role] || member.project_role}
                </span>
              </div>
            </div>
          ))}

          {bottomSpacerHeight > 0 && (
            <div style={{ height: bottomSpacerHeight }} aria-hidden="true" />
          )}
        </div>
      </div>

      <div className="flex items-center justify-center px-2 py-1 text-sm text-muted-foreground">
        <span>{members.length} member{members.length !== 1 && "s"}</span>
      </div>
    </div>
  )
}
