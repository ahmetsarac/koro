"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { use } from "react"
import { Plus, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"

const CATEGORIES = [
  { value: "backlog", label: "Backlog" },
  { value: "unstarted", label: "Unstarted" },
  { value: "started", label: "Started" },
  { value: "completed", label: "Completed" },
  { value: "canceled", label: "Canceled" },
] as const

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

  const [newCategory, setNewCategory] = React.useState<string>("unstarted")
  const [newName, setNewName] = React.useState("")
  const [saving, setSaving] = React.useState(false)

  const [deleteTarget, setDeleteTarget] = React.useState<WorkflowStatusItem | null>(
    null
  )
  const [reassignTo, setReassignTo] = React.useState<string>("")
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
        { cache: "no-store", credentials: "same-origin" }
      )
      if (!res.ok) throw new Error("Failed to load workflow statuses")
      const data: ListResponse = await res.json()
      setGroups(data.groups ?? [])
    } catch (e) {
      setError(e instanceof Error ? e.message : "Load failed")
    } finally {
      setLoading(false)
    }
  }, [orgSlug, projectKey])

  React.useEffect(() => {
    load()
  }, [load])

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault()
    if (!newName.trim()) return
    setSaving(true)
    try {
      const res = await fetch(
        `/api/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          credentials: "same-origin",
          body: JSON.stringify({
            category: newCategory,
            name: newName.trim(),
          }),
        }
      )
      if (!res.ok) {
        const t = await res.text()
        throw new Error(t || "Create failed")
      }
      setNewName("")
      await load()
      router.refresh()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Create failed")
    } finally {
      setSaving(false)
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
      url.searchParams.set("reassign_to", reassignTo)
      const res = await fetch(url.toString(), {
        method: "DELETE",
        credentials: "same-origin",
      })
      if (!res.ok) {
        const t = await res.text()
        throw new Error(t || "Delete failed")
      }
      setDeleteTarget(null)
      setReassignTo("")
      await load()
      router.refresh()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Delete failed")
    } finally {
      setDeleting(false)
    }
  }

  const reassignOptions = allStatuses.filter((s) => s.id !== deleteTarget?.id)

  return (
    <div className="mx-auto flex max-w-2xl flex-col gap-8">
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
        <div className="space-y-6">
          {groups.map((g) => (
            <section
              key={g.category}
              className="rounded-lg border bg-card text-card-foreground"
            >
              <div className="border-b px-4 py-3">
                <h2 className="text-sm font-medium capitalize">{g.category}</h2>
              </div>
              <ul className="divide-y">
                {g.statuses.length === 0 ? (
                  <li className="px-4 py-3 text-sm text-muted-foreground">
                    No statuses in this category.
                  </li>
                ) : (
                  g.statuses.map((s) => (
                    <li
                      key={s.id}
                      className="flex items-center justify-between gap-2 px-4 py-2"
                    >
                      <div>
                        <span className="text-sm font-medium">{s.name}</span>
                        <span className="ml-2 font-mono text-xs text-muted-foreground">
                          {s.slug}
                        </span>
                        {s.is_default ? (
                          <span className="ml-2 text-xs text-muted-foreground">
                            (default)
                          </span>
                        ) : null}
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="text-muted-foreground hover:text-destructive"
                        onClick={() => {
                          setDeleteTarget(s)
                          const firstOther = allStatuses.find((x) => x.id !== s.id)
                          setReassignTo(firstOther?.id ?? "")
                        }}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </li>
                  ))
                )}
              </ul>
            </section>
          ))}
        </div>
      )}

      <form
        onSubmit={handleCreate}
        className="space-y-4 rounded-lg border p-4"
      >
        <h2 className="text-sm font-medium">Add status</h2>
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label>Category</Label>
            <Select value={newCategory} onValueChange={setNewCategory}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CATEGORIES.map((c) => (
                  <SelectItem key={c.value} value={c.value}>
                    {c.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="ws-name">Name</Label>
            <Input
              id="ws-name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="e.g. In review"
            />
          </div>
        </div>
        <Button type="submit" disabled={saving || !newName.trim()}>
          <Plus className="mr-2 h-4 w-4" />
          Add status
        </Button>
      </form>

      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
            setReassignTo("")
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
                setReassignTo("")
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
