"use client"

import * as React from "react"
import Link from "next/link"
import {
  Plus,
  Circle,
  Timer,
  CheckCircle,
  HelpCircle,
} from "lucide-react"
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  closestCenter,
  type DragEndEvent,
  type DragStartEvent,
  type DragOverEvent,
} from "@dnd-kit/core"
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
  arrayMove,
} from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"

import { Skeleton } from "@/components/ui/skeleton"

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

function findColumnForIssue(
  columns: Record<string, IssueListItem[]>,
  issueId: string
): string | null {
  for (const columnId of Object.keys(columns)) {
    if (columns[columnId].some((issue) => issue.issue_id === issueId)) {
      return columnId
    }
  }

  return null
}

export function ProjectKanbanBoard({
  orgSlug,
  projectKey,
}: {
  orgSlug: string
  projectKey: string
}) {
  const [columns, setColumns] = React.useState<Record<string, IssueListItem[]>>(
    {}
  )
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)
  const [activeIssue, setActiveIssue] = React.useState<IssueListItem | null>(
    null
  )

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
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

  const handleDragStart = (event: DragStartEvent) => {
    const issueId = event.active.id as string

    for (const columnId of Object.keys(columns)) {
      const issue = columns[columnId].find((item) => item.issue_id === issueId)
      if (issue) {
        setActiveIssue(issue)
        break
      }
    }
  }

  const handleDragOver = (event: DragOverEvent) => {
    const { active, over } = event
    if (!over) return

    const activeId = active.id as string
    const overId = over.id as string

    const activeColumn = findColumnForIssue(columns, activeId)
    let overColumn = findColumnForIssue(columns, overId)

    if (!overColumn && BOARD_COLUMNS.some((column) => column.id === overId)) {
      overColumn = overId
    }

    if (!activeColumn || !overColumn || activeColumn === overColumn) return

    setColumns((prev) => {
      const activeItems = [...prev[activeColumn]]
      const overItems = [...prev[overColumn]]

      const activeIndex = activeItems.findIndex((item) => item.issue_id === activeId)
      const [movedItem] = activeItems.splice(activeIndex, 1)

      const overIndex = overItems.findIndex((item) => item.issue_id === overId)
      const insertIndex = overIndex >= 0 ? overIndex : overItems.length

      overItems.splice(insertIndex, 0, { ...movedItem, status: overColumn })

      return {
        ...prev,
        [activeColumn]: activeItems,
        [overColumn]: overItems,
      }
    })
  }

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event
    setActiveIssue(null)

    if (!over) return

    const activeId = active.id as string
    const overId = over.id as string

    const activeColumn = findColumnForIssue(columns, activeId)
    if (!activeColumn) return

    if (activeId !== overId) {
      const overColumn = findColumnForIssue(columns, overId)

      if (activeColumn === overColumn && overColumn) {
        setColumns((prev) => {
          const items = [...prev[activeColumn]]
          const oldIndex = items.findIndex((item) => item.issue_id === activeId)
          const newIndex = items.findIndex((item) => item.issue_id === overId)

          return {
            ...prev,
            [activeColumn]: arrayMove(items, oldIndex, newIndex),
          }
        })
      }
    }

    const currentColumn = findColumnForIssue(columns, activeId)
    if (!currentColumn) return

    const currentItems = columns[currentColumn]
    const position = currentItems.findIndex((item) => item.issue_id === activeId)

    try {
      const response = await fetch(`/api/issues/${activeId}/board-position`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status: currentColumn, position }),
        credentials: "same-origin",
      })

      if (!response.ok) {
        await fetchBoard()
      }
    } catch {
      await fetchBoard()
    }
  }

  if (isLoading) {
    return (
      <div className="flex min-h-0 flex-1 gap-4 overflow-x-auto">
        {BOARD_COLUMNS.map((column) => (
          <div
            key={column.id}
            className="flex w-72 shrink-0 flex-col rounded-lg border bg-muted/30"
          >
            <div className="flex items-center gap-2 border-b p-3">
              <Skeleton className="h-4 w-4" />
              <Skeleton className="h-4 w-20" />
            </div>
            <div className="flex-1 space-y-2 p-2">
              {Array.from({ length: 3 }).map((_, index) => (
                <Skeleton key={index} className="h-20 w-full" />
              ))}
            </div>
          </div>
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className="rounded-md border py-12 text-center text-destructive">
        <p>{error}</p>
      </div>
    )
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
    >
      <div className="flex min-h-0 flex-1 gap-4 overflow-x-auto pb-2">
        {BOARD_COLUMNS.map((column) => (
          <BoardColumn
            key={column.id}
            id={column.id}
            label={column.label}
            icon={column.icon}
            issues={columns[column.id] || []}
            orgSlug={orgSlug}
          />
        ))}
      </div>

      <DragOverlay>
        {activeIssue ? (
          <div className="w-64 rotate-3 rounded-md border bg-background p-3 shadow-lg">
            <span className="font-mono text-xs text-muted-foreground">
              {activeIssue.display_key}
            </span>
            <p className="mt-1 line-clamp-2 text-xs font-medium leading-tight">
              {activeIssue.title}
            </p>
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  )
}

function BoardColumn({
  id,
  label,
  icon: ColumnIcon,
  issues,
  orgSlug,
}: {
  id: string
  label: string
  icon: React.ElementType
  issues: IssueListItem[]
  orgSlug: string
}) {
  const issueIds = React.useMemo(() => issues.map((issue) => issue.issue_id), [issues])

  return (
    <div className="flex w-72 shrink-0 flex-col rounded-lg border bg-muted/30">
      <div className="flex items-center gap-2 rounded-t-lg border-b bg-background p-3">
        <ColumnIcon className="h-4 w-4 text-muted-foreground" />
        <span className="text-xs font-medium">{label}</span>
        <span className="ml-auto text-xs text-muted-foreground">{issues.length}</span>
      </div>

      <SortableContext
        id={id}
        items={issueIds}
        strategy={verticalListSortingStrategy}
      >
        <div className="min-h-[100px] flex-1 space-y-2 overflow-y-auto p-2">
          {issues.map((issue) => (
            <SortableBoardCard
              key={issue.issue_id}
              issue={issue}
              orgSlug={orgSlug}
            />
          ))}

          {issues.length === 0 && (
            <div className="py-8 text-center text-xs text-muted-foreground">
              No issues
            </div>
          )}
        </div>
      </SortableContext>

      <div className="p-2">
        <button
          type="button"
          className="flex w-full items-center justify-center rounded-md border border-dashed border-muted-foreground/30 py-2 text-muted-foreground transition-colors hover:border-primary hover:bg-primary/5 hover:text-primary"
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>
    </div>
  )
}

function SortableBoardCard({
  issue,
  orgSlug,
}: {
  issue: IssueListItem
  orgSlug: string
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: issue.issue_id,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...attributes}
      className={`cursor-grab rounded-md border bg-background p-3 shadow-sm transition-opacity active:cursor-grabbing ${
        isDragging ? "opacity-50" : ""
      }`}
    >
      <Link
        href={`/${orgSlug}/issue/${issue.display_key}`}
        className="block space-y-1"
        onClick={(event) => {
          if (isDragging) {
            event.preventDefault()
          }
        }}
      >
        <span className="font-mono text-xs text-muted-foreground">
          {issue.display_key}
        </span>
        <p className="line-clamp-2 text-xs font-medium leading-tight">
          {issue.title}
        </p>
      </Link>
    </div>
  )
}
