"use client"

import * as React from "react"
import { Circle, Timer, CheckCircle, HelpCircle } from "lucide-react"

import { IssueKanbanBoard } from "@/components/issues/issue-kanban-board"

interface IssueListItem {
  issue_id: string
  display_key: string
  title: string
  status: string
}

interface BoardResponse {
  columns: Record<string, IssueListItem[]>
}

const BOARD_COLUMNS = [
  { id: "backlog", label: "Backlog", icon: HelpCircle },
  { id: "todo", label: "Todo", icon: Circle },
  { id: "in_progress", label: "In Progress", icon: Timer },
  { id: "done", label: "Done", icon: CheckCircle },
] as const

export function ProjectKanbanBoard({
  orgSlug,
  projectKey,
  onAddIssue,
}: {
  orgSlug: string
  projectKey: string
  onAddIssue?: (columnId: string) => void
}) {
  const [columns, setColumns] = React.useState<Record<string, IssueListItem[]>>({})
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

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
      setColumns(data.columns)
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load board")
    } finally {
      setIsLoading(false)
    }
  }, [orgSlug, projectKey])

  React.useEffect(() => {
    fetchBoard()
  }, [fetchBoard])

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
          body: JSON.stringify({ status: toColumnId, position }),
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
    <IssueKanbanBoard
      columns={BOARD_COLUMNS}
      itemsByColumn={columns}
      isLoading={isLoading}
      error={error}
      getIssueId={(issue) => issue.issue_id}
      getIssueKey={(issue) => issue.display_key}
      getIssueTitle={(issue) => issue.title}
      getIssueHref={(issue) => `/${orgSlug}/issue/${issue.display_key}`}
      onIssueMove={({ issueId, toColumnId, position }) =>
        handleIssueMove({ issueId, toColumnId, position })
      }
      onReload={fetchBoard}
      onAddIssue={onAddIssue}
    />
  )
}
