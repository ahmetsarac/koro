"use client"

import * as React from "react"
import {
  Circle,
  Timer,
  CheckCircle,
  HelpCircle,
  Ban,
  XCircle,
} from "lucide-react"

import { IssueKanbanBoard } from "@/components/issues/issue-kanban-board"
import { issueDetailHref } from "@/lib/issue-nav"

interface IssueListItem {
  issue_id: string
  display_key: string
  title: string
  status: string
  workflow_status_id: string
  status_name: string
  status_category: string
  is_blocked: boolean
}

interface BoardColumnDef {
  id: string
  name: string
  slug: string
  category: string
  position: number
}

interface BoardResponse {
  column_definitions: BoardColumnDef[]
  items_by_column_id: Record<string, IssueListItem[]>
}

function iconForCategory(category: string): React.ElementType {
  switch (category) {
    case "backlog":
      return HelpCircle
    case "unstarted":
      return Circle
    case "started":
      return Timer
    case "completed":
      return CheckCircle
    case "canceled":
      return XCircle
    default:
      return Circle
  }
}

export function ProjectKanbanBoard({
  orgSlug,
  projectKey,
  projectId,
  onAddIssue,
}: {
  orgSlug: string
  projectKey: string
  /** Same for all columns; enables consistent DnD rules with My Issues board. */
  projectId?: string
  onAddIssue?: (columnId: string) => void
}) {
  const [columnDefinitions, setColumnDefinitions] = React.useState<
    BoardColumnDef[]
  >([])
  const [itemsByColumn, setItemsByColumn] = React.useState<
    Record<string, IssueListItem[]>
  >({})
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  const boardColumns = React.useMemo(
    () =>
      columnDefinitions.map((c) => ({
        id: c.id,
        label: c.name,
        icon: iconForCategory(c.category),
        ...(projectId ? { projectId } : {}),
      })),
    [columnDefinitions, projectId]
  )

  const fetchBoard = React.useCallback(async () => {
    try {
      setIsLoading(true)
      setError(null)

      const response = await fetch(
        `/api/orgs/${orgSlug}/projects/${projectKey}/board`,
        {
          cache: "no-store",
          credentials: "same-origin",
        }
      )

      if (!response.ok) {
        throw new Error("Failed to fetch board")
      }

      const data: BoardResponse = await response.json()
      setColumnDefinitions(data.column_definitions ?? [])
      setItemsByColumn(data.items_by_column_id ?? {})
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load board")
    } finally {
      setIsLoading(false)
    }
  }, [orgSlug, projectKey])

  React.useEffect(() => {
    fetchBoard()
  }, [fetchBoard])

  // Insert newly created issue into the correct board column
  React.useEffect(() => {
    const handler = (event: Event) => {
      const created = (event as CustomEvent).detail as {
        id: string
        display_key: string
        title: string
        status: string
        workflow_status_id: string
        status_name: string
        status_category: string
        is_blocked: boolean
        project_key: string
      }
      if (!created.display_key.startsWith(`${projectKey}-`)) return
      setItemsByColumn((prev) => {
        const colId = created.workflow_status_id
        const next = { ...prev }
        next[colId] = [
          {
            issue_id: created.id,
            display_key: created.display_key,
            title: created.title,
            status: created.status,
            workflow_status_id: created.workflow_status_id,
            status_name: created.status_name,
            status_category: created.status_category,
            is_blocked: created.is_blocked,
          },
          ...(next[colId] || []),
        ]
        return next
      })
    }
    window.addEventListener("koro:issue-created", handler)
    return () => window.removeEventListener("koro:issue-created", handler)
  }, [projectKey])

  const handleIssueMove = React.useCallback(
    async ({
      issueId,
      toColumnId,
      position,
    }: {
      issueId: string
      toColumnId: string
      position: number
    }) => {
      try {
        const response = await fetch(`/api/issues/${issueId}/board-position`, {
          method: "PATCH",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            workflow_status_id: toColumnId,
            position,
          }),
          credentials: "same-origin",
        })

        if (!response.ok) {
          return false
        }
      } catch {
        return false
      }

      return true
    },
    []
  )

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <IssueKanbanBoard
        columns={boardColumns}
        itemsByColumn={itemsByColumn}
        isLoading={isLoading}
        error={error}
        getIssueId={(issue) => issue.issue_id}
        getIssueKey={(issue) => issue.display_key}
        getIssueTitle={(issue) => issue.title}
        getIssueHref={(issue) =>
          issueDetailHref(orgSlug, issue.display_key, {
            from: "project",
            projectKey,
          })
        }
        getIssueProjectId={projectId ? () => projectId : undefined}
        onIssueMove={({ issueId, toColumnId, position }) =>
          handleIssueMove({ issueId, toColumnId, position })
        }
        onReload={fetchBoard}
        onAddIssue={onAddIssue}
      />
    </div>
  )
}
