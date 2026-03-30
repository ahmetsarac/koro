'use client'

import * as React from 'react'
import { useRouter } from 'next/navigation'
import { use } from 'react'
import {
  DndContext,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  SortableContext,
  arrayMove,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical, Plus, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Skeleton } from '@/components/ui/skeleton'

interface WorkflowStatusItem {
  id: string
  category: string
  name: string
  slug: string
  position: number
  is_default: boolean
}

interface WorkflowGroup {
  category: string
  statuses: WorkflowStatusItem[]
}

interface ListResponse {
  groups: WorkflowGroup[]
}

function computeWorkflowReorder(
  prev: WorkflowGroup[],
  activeId: string,
  overId: string
): {
  next: WorkflowGroup[]
  patch: { category: string; statuses: WorkflowStatusItem[] } | null
} {
  for (const g of prev) {
    const oldIndex = g.statuses.findIndex((s) => s.id === activeId)
    const newIndex = g.statuses.findIndex((s) => s.id === overId)
    if (oldIndex === -1 || newIndex === -1) continue
    const newStatuses = arrayMove(g.statuses, oldIndex, newIndex)
    return {
      next: prev.map((x) =>
        x.category === g.category ? { ...x, statuses: newStatuses } : x
      ),
      patch: { category: g.category, statuses: newStatuses },
    }
  }
  return { next: prev, patch: null }
}

function SortableStatusRow({
  status,
  disabled,
  onDelete,
}: {
  status: WorkflowStatusItem
  disabled: boolean
  onDelete: (s: WorkflowStatusItem) => void
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: status.id, disabled })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <li
      ref={setNodeRef}
      style={style}
      className={cn(
        'flex items-center justify-between gap-2 px-4 py-2',
        isDragging && 'relative z-10 bg-card shadow-md'
      )}
    >
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <button
          type="button"
          className={cn(
            'touch-none text-muted-foreground hover:text-foreground',
            disabled && 'pointer-events-none opacity-40'
          )}
          aria-label="Drag to reorder"
          disabled={disabled}
          {...attributes}
          {...listeners}
        >
          <GripVertical className="h-4 w-4 shrink-0" />
        </button>
        <div className="min-w-0">
          <span className="text-sm font-medium">{status.name}</span>
          <span className="ml-2 font-mono text-xs text-muted-foreground">
            {status.slug}
          </span>
          {status.is_default ? (
            <span className="ml-2 text-xs text-muted-foreground">(default)</span>
          ) : null}
        </div>
      </div>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="shrink-0 text-muted-foreground hover:text-destructive"
        disabled={disabled}
        onClick={() => onDelete(status)}
      >
        <Trash2 className="h-4 w-4" />
      </Button>
    </li>
  )
}

export default function WorkflowStatusesSettingsPage({
  params,
}: {
  params: Promise<{ orgSlug: string; projectKey: string }>
}) {
  const { orgSlug, projectKey } = use(params)
  const router = useRouter()
  const [groups, setGroups] = React.useState<WorkflowGroup[]>([])
  const [loading, setLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  const [addingCategory, setAddingCategory] = React.useState<string | null>(null)
  const [draftName, setDraftName] = React.useState('')
  const [savingCategory, setSavingCategory] = React.useState<string | null>(null)
  const [reorderingCategory, setReorderingCategory] = React.useState<string | null>(
    null
  )
  const addInputRef = React.useRef<HTMLInputElement | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    })
  )

  const [deleteTarget, setDeleteTarget] = React.useState<WorkflowStatusItem | null>(
    null
  )
  const [reassignTo, setReassignTo] = React.useState<string>('')
  const [deleting, setDeleting] = React.useState(false)

  const allStatuses = React.useMemo(
    () => groups.flatMap((g) => g.statuses),
    [groups]
  )

  const load = React.useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const res = await fetch(
        `/api/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses`,
        { cache: 'no-store', credentials: 'same-origin' }
      )
      if (!res.ok) throw new Error('Failed to load workflow statuses')
      const data: ListResponse = await res.json()
      setGroups(data.groups ?? [])
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Load failed')
    } finally {
      setLoading(false)
    }
  }, [orgSlug, projectKey])

  React.useEffect(() => {
    load()
  }, [load])

  React.useLayoutEffect(() => {
    if (addingCategory) {
      addInputRef.current?.focus()
    }
  }, [addingCategory])

  function closeAddRow() {
    setAddingCategory(null)
    setDraftName('')
  }

  function toggleAddRow(category: string) {
    if (addingCategory === category) {
      closeAddRow()
    } else {
      setAddingCategory(category)
      setDraftName('')
    }
  }

  async function handleCreateStatus(category: string) {
    const name = draftName.trim()
    if (!name) return
    setSavingCategory(category)
    setError(null)
    try {
      const res = await fetch(
        `/api/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'same-origin',
          body: JSON.stringify({
            category,
            name,
          }),
        }
      )
      if (!res.ok) {
        const t = await res.text()
        throw new Error(t || 'Create failed')
      }
      closeAddRow()
      await load()
      router.refresh()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Create failed')
    } finally {
      setSavingCategory(null)
    }
  }

  async function persistCategoryOrder(
    category: string,
    ordered: WorkflowStatusItem[]
  ) {
    setReorderingCategory(category)
    setError(null)
    try {
      for (let i = 0; i < ordered.length; i++) {
        const s = ordered[i]
        if (s.position === i) continue
        const res = await fetch(
          `/api/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses/${s.id}`,
          {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'same-origin',
            body: JSON.stringify({ position: i }),
          }
        )
        if (!res.ok) {
          const t = await res.text()
          throw new Error(t || 'Reorder failed')
        }
      }
      await load()
      router.refresh()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Reorder failed')
      await load()
    } finally {
      setReorderingCategory(null)
    }
  }

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event
    if (!over || active.id === over.id) return
    const activeId = String(active.id)
    const overId = String(over.id)

    const persistRef: {
      current: { category: string; statuses: WorkflowStatusItem[] } | null
    } = { current: null }
    setGroups((prev) => {
      const { next, patch } = computeWorkflowReorder(prev, activeId, overId)
      persistRef.current = patch
      return next
    })

    const patch = persistRef.current
    if (patch) {
      void persistCategoryOrder(patch.category, patch.statuses)
    }
  }

  async function handleDelete() {
    if (!deleteTarget || !reassignTo) return
    setDeleting(true)
    try {
      const url = new URL(
        `/api/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses/${deleteTarget.id}`,
        window.location.origin
      )
      url.searchParams.set('reassign_to', reassignTo)
      const res = await fetch(url.toString(), {
        method: 'DELETE',
        credentials: 'same-origin',
      })
      if (!res.ok) {
        const t = await res.text()
        throw new Error(t || 'Delete failed')
      }
      setDeleteTarget(null)
      setReassignTo('')
      await load()
      router.refresh()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Delete failed')
    } finally {
      setDeleting(false)
    }
  }

  const reassignOptions = allStatuses.filter((s) => s.id !== deleteTarget?.id)

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h2 className="text-lg font-medium">Workflow statuses</h2>
        <p className="text-sm text-muted-foreground">
          Categories group how work is reported; statuses are your team&apos;s
          steps inside each category.
        </p>
      </div>

      {error ? (
        <p className="text-sm text-destructive">{error}</p>
      ) : null}

      {loading ? (
        <div className="space-y-4">
          <Skeleton className="h-24 w-full" />
          <Skeleton className="h-24 w-full" />
        </div>
      ) : (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <div className="space-y-6">
            {groups.map((g) => {
              const isAdding = addingCategory === g.category
              const savingHere = savingCategory === g.category
              const reorderBusy = reorderingCategory === g.category
              const dndLocked =
                !!savingCategory || !!reorderingCategory || reorderBusy
              const showEmptyHint = g.statuses.length === 0 && !isAdding
              const sortableIds = g.statuses.map((s) => s.id)

              return (
                <section
                  key={g.category}
                  className="rounded-lg border bg-card text-card-foreground"
                >
                  <div className="flex items-center justify-between gap-2 border-b px-4 py-3">
                    <h3 className="text-sm font-medium capitalize">{g.category}</h3>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="shrink-0"
                      aria-label={`Add status to ${g.category}`}
                      title="Add status"
                      disabled={!!reorderingCategory}
                      onClick={() => toggleAddRow(g.category)}
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  <ul className="divide-y">
                    <SortableContext
                      items={sortableIds}
                      strategy={verticalListSortingStrategy}
                    >
                      {g.statuses.map((s) => (
                        <SortableStatusRow
                          key={s.id}
                          status={s}
                          disabled={dndLocked}
                          onDelete={(row) => {
                            setDeleteTarget(row)
                            const firstOther = allStatuses.find(
                              (x) => x.id !== row.id
                            )
                            setReassignTo(firstOther?.id ?? '')
                          }}
                        />
                      ))}
                    </SortableContext>
                    {showEmptyHint ? (
                      <li className="px-4 py-3 text-sm text-muted-foreground">
                        No statuses in this category.
                      </li>
                    ) : null}
                    {isAdding ? (
                      <li className="flex flex-wrap items-center gap-2 px-4 py-2">
                        <Input
                          ref={
                            g.category === addingCategory ? addInputRef : undefined
                          }
                          id={`ws-add-${g.category}`}
                          value={draftName}
                          onChange={(e) => setDraftName(e.target.value)}
                          placeholder="e.g. In review"
                          className="min-w-[12rem] flex-1"
                          disabled={!!savingCategory}
                          onKeyDown={(e) => {
                            if (e.key === 'Escape') closeAddRow()
                            if (
                              e.key === 'Enter' &&
                              draftName.trim() &&
                              !savingHere
                            ) {
                              e.preventDefault()
                              void handleCreateStatus(g.category)
                            }
                          }}
                        />
                        <Button
                          type="button"
                          size="sm"
                          disabled={!draftName.trim() || !!savingCategory}
                          onClick={() => void handleCreateStatus(g.category)}
                        >
                          {savingHere ? 'Adding…' : 'Add'}
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          disabled={!!savingCategory}
                          onClick={closeAddRow}
                        >
                          Cancel
                        </Button>
                      </li>
                    ) : null}
                  </ul>
                  {reorderBusy ? (
                    <p className="border-t px-4 py-2 text-xs text-muted-foreground">
                      Saving order…
                    </p>
                  ) : null}
                </section>
              )
            })}
          </div>
        </DndContext>
      )}

      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
            setReassignTo('')
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete status</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">
            Move all issues currently in &quot;{deleteTarget?.name}&quot; to:
          </p>
          <Select value={reassignTo} onValueChange={setReassignTo}>
            <SelectTrigger>
              <SelectValue placeholder="Choose target status" />
            </SelectTrigger>
            <SelectContent>
              {reassignOptions.map((s) => (
                <SelectItem key={s.id} value={s.id}>
                  {s.name} ({s.slug})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDeleteTarget(null)
                setReassignTo('')
              }}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={!reassignTo || deleting}
              onClick={() => void handleDelete()}
            >
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
