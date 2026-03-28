"use client"

import * as React from "react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Separator } from "@/components/ui/separator"
import { clearAllIssuesCaches } from "@/lib/cache/issues-cache"
import { Grip, X } from "lucide-react"

import type { Issue } from "../data/schema"
import { iconForIssueCategory, priorities } from "../data/data"
import {
  getBulkWorkflowOverlayOptions,
  projectKeyFromDisplayKey,
  type WorkflowBulkOption,
} from "./columns"

export function DataTableSelectionOverlay({
  orgSlug,
  selectedIssues,
  onClearSelection,
  onSuccess,
}: {
  orgSlug: string
  selectedIssues: Issue[]
  onClearSelection: () => void
  onSuccess: () => Promise<void>
}) {
  const selectedCount = selectedIssues.length
  const [actionsOpen, setActionsOpen] = React.useState(false)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [statusOptions, setStatusOptions] = React.useState<WorkflowBulkOption[]>(
    []
  )

  const projectKeysSorted = React.useMemo(() => {
    const keys = selectedIssues.map((i) =>
      projectKeyFromDisplayKey(i.display_key).trim().toUpperCase()
    )
    return [...new Set(keys)].sort((a, b) => a.localeCompare(b))
  }, [selectedIssues])

  React.useEffect(() => {
    if (!actionsOpen || projectKeysSorted.length === 0) {
      setStatusOptions([])
      return
    }
    let cancelled = false
    getBulkWorkflowOverlayOptions(orgSlug, projectKeysSorted)
      .then((opts) => {
        if (!cancelled) setStatusOptions(opts)
      })
      .catch(() => {
        if (!cancelled) setStatusOptions([])
      })
    return () => {
      cancelled = true
    }
  }, [actionsOpen, orgSlug, projectKeysSorted])

  async function handleArchive() {
    if (busy || selectedIssues.length === 0) return
    setBusy(true)
    setError(null)
    try {
      const res = await fetch("/api/my-issues/bulk", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({
          archive: { issue_ids: selectedIssues.map((i) => i.id) },
        }),
      })
      const data = await res.json().catch(() => ({})) as { message?: string }
      if (!res.ok) {
        throw new Error(
          typeof data?.message === "string" && data.message.length > 0
            ? data.message
            : "Could not archive issues."
        )
      }
      clearAllIssuesCaches()
      onClearSelection()
      setActionsOpen(false)
      await onSuccess()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Archive failed.")
    } finally {
      setBusy(false)
    }
  }

  async function handleSetPriority(priority: string) {
    if (busy || selectedIssues.length === 0) return
    setBusy(true)
    setError(null)
    try {
      const res = await fetch("/api/my-issues/bulk", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({
          set_priority: {
            issue_ids: selectedIssues.map((i) => i.id),
            priority,
          },
        }),
      })
      const data = await res.json().catch(() => ({})) as { message?: string }
      if (!res.ok) {
        throw new Error(
          typeof data?.message === "string" && data.message.length > 0
            ? data.message
            : "Could not update priority."
        )
      }
      clearAllIssuesCaches()
      onClearSelection()
      setActionsOpen(false)
      await onSuccess()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Priority update failed.")
    } finally {
      setBusy(false)
    }
  }

  async function handleSetStatus(workflowStatusSlug: string) {
    if (busy || selectedIssues.length === 0) return
    setBusy(true)
    setError(null)
    try {
      const res = await fetch("/api/my-issues/bulk", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({
          set_status: {
            issue_ids: selectedIssues.map((i) => i.id),
            workflow_status_slug: workflowStatusSlug,
          },
        }),
      })
      const data = await res.json().catch(() => ({})) as { message?: string }
      if (!res.ok) {
        throw new Error(
          typeof data?.message === "string" && data.message.length > 0
            ? data.message
            : "Could not update status."
        )
      }
      clearAllIssuesCaches()
      onClearSelection()
      setActionsOpen(false)
      await onSuccess()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Status update failed.")
    } finally {
      setBusy(false)
    }
  }

  if (selectedCount === 0) return null

  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-4 z-20 flex flex-col items-center gap-1 px-4">
      {error && (
        <p className="pointer-events-auto max-w-md rounded-md border border-destructive/30 bg-destructive/10 px-2 py-1 text-center text-xs text-destructive">
          {error}
        </p>
      )}
      <div className="pointer-events-auto flex items-center gap-1 rounded-lg border border-white/10 bg-sidebar p-1.5 text-white shadow-[0_4px_8px_rgba(0,0,0,0.28)] backdrop-blur-xl">
        <div className="flex items-center gap-0">
          <div className="flex h-7 items-center rounded-l-md border border-r-0 border-dashed border-sidebar-border bg-sidebar px-2 text-xs font-medium tracking-tight text-foreground">
            {selectedCount} selected
          </div>
          <Button
            size="icon"
            className="size-7 rounded-l-none rounded-r-md border border-dashed border-sidebar-border bg-sidebar text-foreground hover:bg-background/90 hover:text-foreground"
            onClick={onClearSelection}
          >
            <X />
            <span className="sr-only">Clear selection</span>
          </Button>
        </div>

        <Separator
          orientation="vertical"
          className="mx-1 h-5 data-vertical:self-auto"
        />

        <DropdownMenu
          open={actionsOpen}
          onOpenChange={(o) => {
            setActionsOpen(o)
            if (!o) setError(null)
          }}
        >
          <DropdownMenuTrigger asChild>
            <Button className="rounded-md border border-sidebar-border bg-sidebar px-3 text-foreground hover:bg-background/90 hover:text-foreground">
              <Grip />
              Actions
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align="center"
            side="top"
            sideOffset={12}
            className="min-w-48"
          >
            <DropdownMenuSub>
              <DropdownMenuSubTrigger
                disabled={busy}
                className="data-[disabled]:opacity-50"
              >
                Change status
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="max-h-64 overflow-y-auto">
                {statusOptions.length === 0 && (
                  <div className="px-2 py-1.5 text-xs text-muted-foreground">
                    No statuses available.
                  </div>
                )}
                {statusOptions.map((s) => {
                  const OptIcon = iconForIssueCategory(s.category)
                  const selectable = s.bulkSelectable && !busy
                  return (
                    <DropdownMenuItem
                      key={s.slug}
                      disabled={!selectable}
                      title={
                        s.bulkSelectable
                          ? undefined
                          : "Not available in all selected projects"
                      }
                      onSelect={(ev) => {
                        ev.preventDefault()
                        if (!s.bulkSelectable) return
                        void handleSetStatus(s.slug)
                      }}
                      className="gap-2"
                    >
                      <OptIcon className="size-3.5 text-muted-foreground" />
                      <span>{s.name}</span>
                    </DropdownMenuItem>
                  )
                })}
              </DropdownMenuSubContent>
            </DropdownMenuSub>
            <DropdownMenuSub>
              <DropdownMenuSubTrigger
                disabled={busy}
                className="data-[disabled]:opacity-50"
              >
                Change priority
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="max-h-64 overflow-y-auto">
                {priorities.map((p) => {
                  const Icon = p.icon
                  return (
                    <DropdownMenuItem
                      key={p.value}
                      disabled={busy}
                      onSelect={(ev) => {
                        ev.preventDefault()
                        void handleSetPriority(p.value)
                      }}
                      className="gap-2"
                    >
                      <Icon className="size-3.5 text-muted-foreground" />
                      <span>{p.label}</span>
                    </DropdownMenuItem>
                  )
                })}
              </DropdownMenuSubContent>
            </DropdownMenuSub>
            <DropdownMenuItem
              variant="destructive"
              disabled={busy}
              onSelect={(ev) => {
                ev.preventDefault()
                void handleArchive()
              }}
            >
              {busy ? "Working…" : "Archive"}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
