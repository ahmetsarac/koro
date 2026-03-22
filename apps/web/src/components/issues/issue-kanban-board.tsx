"use client"

import * as React from "react"
import Link from "next/link"
import { Ban, MoreHorizontal, Plus } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  useDroppable,
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

export interface KanbanColumn {
  id: string
  label: string
  icon?: React.ElementType
  /** When set (e.g. My Issues), only issues from this project may drop into this column. */
  projectId?: string
}

export interface KanbanIssue {
  status: string
  is_blocked?: boolean
  project_id?: string
}

export interface IssueMoveParams<TIssue extends KanbanIssue> {
  issue: TIssue
  issueId: string
  fromColumnId: string
  toColumnId: string
  position: number
  columns: Record<string, TIssue[]>
}

interface IssueKanbanBoardProps<TIssue extends KanbanIssue> {
  columns: readonly KanbanColumn[]
  itemsByColumn: Record<string, TIssue[]>
  isLoading?: boolean
  error?: string | null
  emptyMessage?: string
  getIssueId: (issue: TIssue) => string
  getIssueKey: (issue: TIssue) => string
  getIssueTitle: (issue: TIssue) => string
  getIssueHref?: (issue: TIssue) => string
  onIssueMove?: (
    params: IssueMoveParams<TIssue>
  ) => Promise<boolean | void> | boolean | void
  onReload?: () => Promise<void> | void
  onAddIssue?: (columnId: string) => void
  /** Used with `KanbanColumn.projectId` to block cross-project drops on My Issues. */
  getIssueProjectId?: (issue: TIssue) => string | undefined
  /** My Issues board: column header menu to hide a column. */
  onHideColumn?: (columnId: string) => void
}

function normalizeColumns<TIssue extends KanbanIssue>(
  columns: readonly KanbanColumn[],
  itemsByColumn: Record<string, TIssue[]>
): Record<string, TIssue[]> {
  return columns.reduce<Record<string, TIssue[]>>((acc, column) => {
    acc[column.id] = itemsByColumn[column.id] ? [...itemsByColumn[column.id]] : []
    return acc
  }, {})
}

function findColumnForIssue<TIssue extends KanbanIssue>(
  columns: Record<string, TIssue[]>,
  issueId: string,
  getIssueId: (issue: TIssue) => string
): string | null {
  for (const columnId of Object.keys(columns)) {
    if (columns[columnId].some((issue) => getIssueId(issue) === issueId)) {
      return columnId
    }
  }

  return null
}

export function IssueKanbanBoard<TIssue extends KanbanIssue>({
  columns,
  itemsByColumn,
  isLoading = false,
  error = null,
  emptyMessage = "No issues",
  getIssueId,
  getIssueKey,
  getIssueTitle,
  getIssueHref,
  onIssueMove,
  onReload,
  onAddIssue,
  getIssueProjectId,
  onHideColumn,
}: IssueKanbanBoardProps<TIssue>) {
  const normalizedColumns = React.useMemo(
    () => normalizeColumns(columns, itemsByColumn),
    [columns, itemsByColumn]
  )
  const [localColumns, setLocalColumns] = React.useState(normalizedColumns)
  const [activeIssue, setActiveIssue] = React.useState<TIssue | null>(null)
  const localColumnsRef = React.useRef(localColumns)
  const dragSourceColumnIdRef = React.useRef<string | null>(null)
  const isInteractive = Boolean(onIssueMove)

  React.useEffect(() => {
    setLocalColumns(normalizedColumns)
    localColumnsRef.current = normalizedColumns
  }, [normalizedColumns])

  const updateColumns = React.useCallback(
    (
      updater:
        | Record<string, TIssue[]>
        | ((prev: Record<string, TIssue[]>) => Record<string, TIssue[]>)
    ) => {
      setLocalColumns((prev) => {
        const next = typeof updater === "function" ? updater(prev) : updater
        localColumnsRef.current = next
        return next
      })
    },
    []
  )

  const resetColumns = React.useCallback(() => {
    updateColumns(normalizedColumns)
  }, [normalizedColumns, updateColumns])

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  )

  const handleDragStart = (event: DragStartEvent) => {
    if (!isInteractive) return

    const issueId = event.active.id as string
    dragSourceColumnIdRef.current = null

    for (const columnId of Object.keys(localColumnsRef.current)) {
      const issue = localColumnsRef.current[columnId].find(
        (item) => getIssueId(item) === issueId
      )
      if (issue) {
        dragSourceColumnIdRef.current = columnId
        setActiveIssue(issue)
        break
      }
    }
  }

  const handleDragOver = (event: DragOverEvent) => {
    if (!isInteractive) return

    const { active, over } = event
    if (!over) return

    const activeId = active.id as string
    const overId = over.id as string
    const currentColumns = localColumnsRef.current

    const activeColumn = findColumnForIssue(currentColumns, activeId, getIssueId)
    let overColumn = findColumnForIssue(currentColumns, overId, getIssueId)

    if (!overColumn && columns.some((column) => column.id === overId)) {
      overColumn = overId
    }

    if (!activeColumn || !overColumn || activeColumn === overColumn) return

    const activeIssueRow = currentColumns[activeColumn]?.find(
      (item) => getIssueId(item) === activeId
    )
    const targetCol = columns.find((c) => c.id === overColumn)
    if (
      targetCol?.projectId &&
      getIssueProjectId &&
      activeIssueRow &&
      getIssueProjectId(activeIssueRow) !== targetCol.projectId
    ) {
      return
    }

    updateColumns((prev) => {
      const activeItems = [...prev[activeColumn]]
      const overItems = [...prev[overColumn]]

      const activeIndex = activeItems.findIndex(
        (item) => getIssueId(item) === activeId
      )
      const [movedItem] = activeItems.splice(activeIndex, 1)

      const overIndex = overItems.findIndex((item) => getIssueId(item) === overId)
      const insertIndex = overIndex >= 0 ? overIndex : overItems.length

      overItems.splice(insertIndex, 0, {
        ...movedItem,
        status: overColumn,
      })

      return {
        ...prev,
        [activeColumn]: activeItems,
        [overColumn]: overItems,
      }
    })
  }

  const handleDragEnd = async (event: DragEndEvent) => {
    setActiveIssue(null)

    if (!isInteractive) return

    const { active, over } = event
    if (!over) {
      resetColumns()
      return
    }

    const activeId = active.id as string
    const overId = over.id as string
    const startColumns = localColumnsRef.current

    // Use source column stored at drag start; ref may already be updated by handleDragOver
    const fromColumnId = dragSourceColumnIdRef.current ?? findColumnForIssue(startColumns, activeId, getIssueId)
    if (!fromColumnId) return

    let nextColumns = startColumns

    if (activeId !== overId) {
      const overColumnId = findColumnForIssue(startColumns, overId, getIssueId)

      if (fromColumnId === overColumnId && overColumnId) {
        nextColumns = {
          ...startColumns,
          [fromColumnId]: arrayMove(
            startColumns[fromColumnId],
            startColumns[fromColumnId].findIndex(
              (item) => getIssueId(item) === activeId
            ),
            startColumns[fromColumnId].findIndex(
              (item) => getIssueId(item) === overId
            )
          ),
        }

        updateColumns(nextColumns)
      }
    }

    // Cross-column move: ref was updated in handleDragOver; use latest for toColumnId
    const currentColumns = localColumnsRef.current
    let toColumnId = findColumnForIssue(currentColumns, activeId, getIssueId)
    // Fallback: dropped on column droppable (overId is column id) but ref not yet updated
    if (!toColumnId && columns.some((c) => c.id === overId)) {
      toColumnId = overId
    }
    if (!toColumnId) return

    const nextItems = currentColumns[toColumnId] ?? []
    const position = nextItems.findIndex((item) => getIssueId(item) === activeId)
    let issue = position >= 0 ? nextItems[position] : undefined
    // Fallback: get moved issue from source column when ref not yet updated
    if (!issue) {
      const fromItems = startColumns[fromColumnId] ?? []
      issue = fromItems.find((item) => getIssueId(item) === activeId)
    }
    if (!issue) return

    const targetColMeta = columns.find((c) => c.id === toColumnId)
    if (
      targetColMeta?.projectId &&
      getIssueProjectId &&
      getIssueProjectId(issue) !== targetColMeta.projectId
    ) {
      resetColumns()
      return
    }

    const resolvedPosition = position >= 0 ? position : 0

    try {
      const result = await onIssueMove?.({
        issue: { ...issue, status: toColumnId },
        issueId: activeId,
        fromColumnId,
        toColumnId,
        position: resolvedPosition,
        columns: currentColumns,
      })

      if (result === false) {
        resetColumns()
        await onReload?.()
      }
    } catch {
      resetColumns()
      await onReload?.()
    }
  }

  if (isLoading) {
    const skeletonColumns =
      columns.length > 0
        ? columns
        : ([1, 2, 3, 4, 5] as const).map((n) => ({
            id: `sk-${n}`,
            label: "",
          }))
    return (
      <div className="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <div className="flex min-h-0 flex-1 gap-4 overflow-x-auto overflow-y-hidden pb-2">
          {skeletonColumns.map((column) => (
            <div
              key={column.id}
              className="flex h-full max-h-full min-h-0 w-72 shrink-0 flex-col rounded-lg border bg-muted/30"
            >
              <div className="flex shrink-0 items-center gap-2 border-b p-3">
                <Skeleton className="h-4 w-4" />
                <Skeleton className="h-4 w-20" />
              </div>
              <div className="min-h-0 flex-1 space-y-2 overflow-y-auto p-2">
                {Array.from({ length: 3 }).map((_, index) => (
                  <Skeleton key={index} className="h-20 w-full shrink-0" />
                ))}
              </div>
            </div>
          ))}
        </div>
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
      <div className="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <div className="flex min-h-0 flex-1 gap-4 overflow-x-auto overflow-y-hidden pb-2">
          {columns.map((column) => (
            <BoardColumn
              key={column.id}
              id={column.id}
              label={column.label}
              icon={column.icon}
              columnProjectId={column.projectId}
              activeDragIssue={activeIssue}
              getIssueProjectId={getIssueProjectId}
              issues={localColumns[column.id] || []}
              emptyMessage={emptyMessage}
              getIssueId={getIssueId}
              getIssueKey={getIssueKey}
              getIssueTitle={getIssueTitle}
              getIssueHref={getIssueHref}
              onAddIssue={onAddIssue}
              isInteractive={isInteractive}
              onHideColumn={onHideColumn}
            />
          ))}
        </div>
      </div>

      <DragOverlay>
        {activeIssue ? (
          <div className="w-64 rotate-3 rounded-md border bg-background p-3 shadow-lg">
            <span className="font-mono text-xs text-muted-foreground">
              {getIssueKey(activeIssue)}
            </span>
            <p className="mt-1 line-clamp-2 text-xs font-medium leading-tight">
              {getIssueTitle(activeIssue)}
            </p>
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  )
}

interface BoardColumnProps<TIssue extends KanbanIssue> {
  id: string
  label: string
  icon?: React.ElementType
  columnProjectId?: string
  activeDragIssue: TIssue | null
  getIssueProjectId?: (issue: TIssue) => string | undefined
  issues: TIssue[]
  emptyMessage: string
  getIssueId: (issue: TIssue) => string
  getIssueKey: (issue: TIssue) => string
  getIssueTitle: (issue: TIssue) => string
  getIssueHref?: (issue: TIssue) => string
  onAddIssue?: (columnId: string) => void
  isInteractive: boolean
  onHideColumn?: (columnId: string) => void
}

function BoardColumn<TIssue extends KanbanIssue>({
  id,
  label,
  icon: ColumnIcon,
  columnProjectId,
  activeDragIssue,
  getIssueProjectId,
  issues,
  emptyMessage,
  getIssueId,
  getIssueKey,
  getIssueTitle,
  getIssueHref,
  onAddIssue,
  isInteractive,
  onHideColumn,
}: BoardColumnProps<TIssue>) {
  const issueIds = React.useMemo(() => issues.map(getIssueId), [issues, getIssueId])
  const dropDisabled = Boolean(
    columnProjectId &&
      activeDragIssue &&
      getIssueProjectId &&
      getIssueProjectId(activeDragIssue) !== columnProjectId
  )
  const { setNodeRef: setDroppableRef, isOver } = useDroppable({
    id,
    disabled: dropDisabled,
  })

  return (
    <div className="flex h-full max-h-full min-h-0 w-72 shrink-0 flex-col rounded-lg border bg-muted/30">
      <div className="flex shrink-0 items-center gap-2 rounded-t-lg border-b bg-background px-2 py-2">
        {ColumnIcon ? (
          <ColumnIcon className="h-4 w-4 shrink-0 text-muted-foreground" />
        ) : null}
        <div className="flex min-w-0 flex-1 items-center gap-1.5">
          <span className="min-w-0 truncate text-xs font-medium">{label}</span>
          <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
            {issues.length}
          </span>
        </div>
        {onHideColumn ? (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="h-7 w-7 shrink-0"
                aria-label="Column options"
                onPointerDown={(e) => e.stopPropagation()}
              >
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44">
              <DropdownMenuItem
                className="text-xs"
                onClick={() => onHideColumn(id)}
              >
                Hide column
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ) : null}
      </div>

      <SortableContext
        id={id}
        items={issueIds}
        strategy={verticalListSortingStrategy}
      >
        <div
          ref={setDroppableRef}
          className={`min-h-0 flex-1 space-y-2 overflow-y-auto p-2 ${isOver && !dropDisabled ? "rounded-md bg-muted/50" : ""} ${dropDisabled ? "opacity-40" : ""}`}
        >
          {issues.map((issue) => (
            <SortableBoardCard
              key={getIssueId(issue)}
              issue={issue}
              getIssueId={getIssueId}
              getIssueKey={getIssueKey}
              getIssueTitle={getIssueTitle}
              getIssueHref={getIssueHref}
              isInteractive={isInteractive}
            />
          ))}

          {issues.length === 0 && (
            <div className="py-8 text-center text-xs text-muted-foreground">
              {emptyMessage}
            </div>
          )}
        </div>
      </SortableContext>

      <div className="shrink-0 p-2">
        <button
          type="button"
          onClick={() => onAddIssue?.(id)}
          className="flex w-full items-center justify-center rounded-md border border-dashed border-muted-foreground/30 py-2 text-muted-foreground transition-colors hover:border-primary hover:bg-primary/5 hover:text-primary"
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>
    </div>
  )
}

interface SortableBoardCardProps<TIssue extends KanbanIssue> {
  issue: TIssue
  getIssueId: (issue: TIssue) => string
  getIssueKey: (issue: TIssue) => string
  getIssueTitle: (issue: TIssue) => string
  getIssueHref?: (issue: TIssue) => string
  isInteractive: boolean
}

function SortableBoardCard<TIssue extends KanbanIssue>({
  issue,
  getIssueId,
  getIssueKey,
  getIssueTitle,
  getIssueHref,
  isInteractive,
}: SortableBoardCardProps<TIssue>) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: getIssueId(issue),
    disabled: !isInteractive,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  const href = getIssueHref?.(issue)

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...(isInteractive ? listeners : {})}
      {...(isInteractive ? attributes : {})}
      className={`rounded-md border bg-background p-3 shadow-sm transition-opacity ${
        isInteractive ? "cursor-grab active:cursor-grabbing" : ""
      } ${isDragging ? "opacity-50" : ""}`}
    >
      {href ? (
        <Link
          href={href}
          className="block space-y-1"
          onClick={(event) => {
            if (isDragging) {
              event.preventDefault()
            }
          }}
        >
          <span className="font-mono text-xs text-muted-foreground">
            {getIssueKey(issue)}
          </span>
          <p className="line-clamp-2 text-xs font-medium leading-tight">
            {getIssueTitle(issue)}
          </p>
          {issue.is_blocked ? (
            <span className="mt-1 inline-flex items-center gap-1 rounded bg-destructive/10 px-1.5 py-0.5 text-[10px] font-medium text-destructive">
              <Ban className="h-3 w-3" />
              Blocked
            </span>
          ) : null}
        </Link>
      ) : (
        <div className="space-y-1">
          <span className="font-mono text-xs text-muted-foreground">
            {getIssueKey(issue)}
          </span>
          <p className="line-clamp-2 text-xs font-medium leading-tight">
            {getIssueTitle(issue)}
          </p>
          {issue.is_blocked ? (
            <span className="mt-1 inline-flex items-center gap-1 rounded bg-destructive/10 px-1.5 py-0.5 text-[10px] font-medium text-destructive">
              <Ban className="h-3 w-3" />
              Blocked
            </span>
          ) : null}
        </div>
      )}
    </div>
  )
}
