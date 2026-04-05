"use client"

import * as React from "react"
import Link from "next/link"
import { type ColumnDef } from "@tanstack/react-table"
import { Ban, Check } from "lucide-react"

import { Checkbox } from "@/components/ui/checkbox"
import { Badge } from "@/components/ui/badge"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

import { priorities, iconForIssueCategory } from "../data/data"
import { type Issue } from "../data/schema"
import { DataTableColumnHeader } from "./data-table-column-header"
import { DataTableRowActions } from "./data-table-row-actions"
import { updateIssueInCaches } from "@/lib/cache/issues-cache"
import { issueDetailHref } from "@/lib/issue-nav"

export interface WorkflowOption {
  id: string
  category: string
  name: string
  slug: string
  position: number
  is_default: boolean
}

/** Multi-project selection overlay: union of statuses; only `bulkSelectable` rows apply to every project. */
export type WorkflowBulkOption = WorkflowOption & { bulkSelectable: boolean }

/** Matches `list_workflow_statuses_for_project_ordered` in the API (backlog → todo → …). */
const WORKFLOW_CATEGORY_ORDER: Record<string, number> = {
  backlog: 0,
  unstarted: 1,
  started: 2,
  completed: 3,
  canceled: 4,
}

function categoryOrderRank(category: string): number {
  return WORKFLOW_CATEGORY_ORDER[category] ?? 100
}

function compareWorkflowOptionsByCategory(a: WorkflowOption, b: WorkflowOption): number {
  const ra = categoryOrderRank(a.category)
  const rb = categoryOrderRank(b.category)
  if (ra !== rb) {
    if (ra >= 100 && rb >= 100) {
      return a.category.localeCompare(b.category)
    }
    return ra - rb
  }
  if (a.position !== b.position) return a.position - b.position
  return a.slug.localeCompare(b.slug)
}

const workflowOptionsCache = new Map<string, WorkflowOption[]>()
const workflowOptionsInflight = new Map<string, Promise<WorkflowOption[]>>()

function workflowCacheKey(orgSlug: string, projectKey: string) {
  return `${orgSlug}\0${projectKey}`
}

export function projectKeyFromDisplayKey(displayKey: string): string {
  const i = displayKey.lastIndexOf("-")
  return i >= 0 ? displayKey.slice(0, i) : displayKey
}

export async function getWorkflowStatusOptions(
  orgSlug: string,
  projectKey: string
): Promise<WorkflowOption[]> {
  const normalizedKey = projectKey.trim().toUpperCase()
  const k = workflowCacheKey(orgSlug, normalizedKey)
  const hit = workflowOptionsCache.get(k)
  if (hit) return hit
  const inflight = workflowOptionsInflight.get(k)
  if (inflight) return inflight

  const p = fetch(
    `/api/orgs/${orgSlug}/projects/${normalizedKey}/workflow-statuses`,
    { cache: "no-store", credentials: "same-origin" }
  )
    .then(async (response) => {
      if (!response.ok) {
        throw new Error(`workflow-statuses ${response.status}`)
      }
      const data = (await response.json()) as {
        groups?: { category: string; statuses: WorkflowOption[] }[]
      }
      const flat: WorkflowOption[] = []
      for (const g of data.groups ?? []) {
        for (const s of g.statuses ?? []) {
          flat.push({ ...s, category: s.category || g.category })
        }
      }
      flat.sort(compareWorkflowOptionsByCategory)
      workflowOptionsCache.set(k, flat)
      return flat
    })
    .finally(() => {
      workflowOptionsInflight.delete(k)
    })

  workflowOptionsInflight.set(k, p)
  return p
}

/** Union of workflow statuses across projects; `bulkSelectable` only if the slug exists in every project. */
export async function getBulkWorkflowOverlayOptions(
  orgSlug: string,
  projectKeys: string[]
): Promise<WorkflowBulkOption[]> {
  const uniqueSorted = [
    ...new Set(projectKeys.map((k) => k.trim().toUpperCase())),
  ].sort((a, b) => a.localeCompare(b))
  if (uniqueSorted.length === 0) return []
  const lists = await Promise.all(
    uniqueSorted.map((k) => getWorkflowStatusOptions(orgSlug, k))
  )
  if (uniqueSorted.length === 1) {
    return lists[0]!.map((o) => ({ ...o, bulkSelectable: true }))
  }

  const n = lists.length
  const slugProjectCount = new Map<string, number>()
  const slugRepresentative = new Map<string, WorkflowOption>()

  for (let pi = 0; pi < n; pi++) {
    const seen = new Set<string>()
    for (const o of lists[pi]!) {
      if (seen.has(o.slug)) continue
      seen.add(o.slug)
      slugProjectCount.set(o.slug, (slugProjectCount.get(o.slug) ?? 0) + 1)
      if (!slugRepresentative.has(o.slug)) {
        slugRepresentative.set(o.slug, o)
      }
    }
  }

  const out: WorkflowBulkOption[] = []
  for (const rep of slugRepresentative.values()) {
    const count = slugProjectCount.get(rep.slug) ?? 0
    out.push({
      ...rep,
      bulkSelectable: count === n,
    })
  }
  out.sort(compareWorkflowOptionsByCategory)
  return out
}

async function updateIssuePriority(
  orgSlug: string,
  displayKey: string,
  priority: string
): Promise<boolean> {
  const response = await fetch(`/api/orgs/${orgSlug}/issues/${displayKey}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ priority }),
  })

  return response.ok
}

async function updateIssueStatus(
  issueId: string,
  workflowStatusId: string
): Promise<{ ok: boolean; status?: string }> {
  const response = await fetch(`/api/issues/${issueId}/status`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ workflow_status_id: workflowStatusId }),
  })
  if (!response.ok) return { ok: false }
  try {
    const j = (await response.json()) as { status?: string }
    return { ok: true, status: j.status }
  } catch {
    return { ok: true }
  }
}

export const createColumns = (orgSlug: string): ColumnDef<Issue>[] => [
  {
    id: "select",
    size: 28,
    header: ({ table }) => (
      <Checkbox
        checked={
          table.getIsAllPageRowsSelected() ||
          (table.getIsSomePageRowsSelected() && "indeterminate")
        }
        onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
        aria-label="Select all"
        className="translate-y-[2px]"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label="Select row"
        className="translate-y-[2px]"
      />
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "display_key",
    id: "id",
    size: 100,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Issue" />
    ),
    cell: ({ row }) => (
      <Link
        href={issueDetailHref(orgSlug, row.original.display_key, {
          from: "my-issues",
        })}
        className="w-[80px] font-mono text-xs hover:underline"
      >
        {row.original.display_key}
      </Link>
    ),
    enableSorting: true,
    enableHiding: false,
  },
  {
    accessorKey: "title",
    size: 450,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Title" />
    ),
    cell: ({ row }) => (
      <Link
        href={issueDetailHref(orgSlug, row.original.display_key, {
          from: "my-issues",
        })}
        className="max-w-[500px] truncate font-medium hover:underline"
      >
        {row.getValue("title")}
      </Link>
    ),
  },
  {
    id: "status",
    accessorKey: "workflow_status_id",
    size: 160,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Status" />
    ),
    cell: ({ row }) => {
      const issue = row.original
      const [menuOpen, setMenuOpen] = React.useState(false)
      const [options, setOptions] = React.useState<WorkflowOption[] | null>(
        null
      )
      const [optionsLoading, setOptionsLoading] = React.useState(false)
      const [currentWid, setCurrentWid] = React.useState(
        issue.workflow_status_id
      )
      const [currentName, setCurrentName] = React.useState(issue.status_name)
      const [currentCategory, setCurrentCategory] = React.useState(
        issue.status_category
      )
      const [isUpdating, setIsUpdating] = React.useState(false)

      React.useEffect(() => {
        setCurrentWid(issue.workflow_status_id)
        setCurrentName(issue.status_name)
        setCurrentCategory(issue.status_category)
      }, [
        issue.workflow_status_id,
        issue.status_name,
        issue.status_category,
      ])

      const StatusIcon = iconForIssueCategory(currentCategory)

      const handleOpenChange = (open: boolean) => {
        setMenuOpen(open)
        if (open && options === null) {
          setOptionsLoading(true)
          const pk = projectKeyFromDisplayKey(issue.display_key)
          getWorkflowStatusOptions(orgSlug, pk)
            .then(setOptions)
            .finally(() => setOptionsLoading(false))
        }
      }

      const handleStatusChange = async (newId: string) => {
        if (isUpdating || newId === currentWid) return
        setIsUpdating(true)
        try {
          const res = await updateIssueStatus(issue.id, newId)
          if (res.ok) {
            const opt = options?.find((o) => o.id === newId)
            const slug = res.status ?? opt?.slug ?? issue.status
            setCurrentWid(newId)
            setCurrentName(opt?.name ?? issue.status_name)
            setCurrentCategory(opt?.category ?? issue.status_category)
            updateIssueInCaches(issue.display_key, {
              status: slug,
              workflow_status_id: newId,
              status_name: opt?.name ?? issue.status_name,
              status_category: opt?.category ?? issue.status_category,
            })
          }
        } finally {
          setIsUpdating(false)
        }
      }

      return (
        <div className="inline-flex flex-wrap items-center gap-1.5">
          <DropdownMenu open={menuOpen} onOpenChange={handleOpenChange}>
            <DropdownMenuTrigger asChild disabled={isUpdating}>
              <button type="button" className="focus:outline-none">
                <Badge
                  variant="outline"
                  className="flex w-fit cursor-pointer items-center justify-start gap-2 hover:bg-muted/5"
                >
                  <StatusIcon className="size-3.5 text-muted-foreground" />
                  <span className="text-xs">{currentName}</span>
                </Badge>
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="start"
              className="max-h-72 overflow-y-auto"
            >
              {optionsLoading ? (
                <DropdownMenuItem disabled className="text-xs">
                  Loading…
                </DropdownMenuItem>
              ) : (options?.length ?? 0) === 0 ? (
                <DropdownMenuItem disabled className="text-xs">
                  No statuses
                </DropdownMenuItem>
              ) : (
                (options ?? []).map((o) => {
                  const OptIcon = iconForIssueCategory(o.category)
                  const isActive = o.id === currentWid
                  return (
                    <DropdownMenuItem
                      key={o.id}
                      onClick={() => void handleStatusChange(o.id)}
                      className="gap-2 text-xs"
                    >
                      <OptIcon className="h-3.5 w-3.5" />
                      {o.name}
                      {isActive && (
                        <Check className="ml-auto h-3.5 w-3.5" />
                      )}
                    </DropdownMenuItem>
                  )
                })
              )}
            </DropdownMenuContent>
          </DropdownMenu>
          {issue.is_blocked ? (
            <Link
              href={issueDetailHref(orgSlug, issue.display_key, {
                from: "my-issues",
              })}
              className="inline-flex"
            >
              <Badge
                variant="outline"
                className="gap-1 border-destructive/30 bg-destructive/10 text-[10px] text-destructive"
                title="Blocked by another issue"
              >
                <Ban className="size-3" />
                Blocked
              </Badge>
            </Link>
          ) : null}
        </div>
      )
    },
    filterFn: (row, _id, value) => {
      const v = value as string[] | undefined
      if (!v?.length) return true
      return v.includes(row.original.status)
    },
  },
  {
    id: "relations",
    accessorFn: () => "",
    size: 0,
    minSize: 0,
    maxSize: 0,
    header: () => null,
    cell: () => null,
    enableSorting: false,
    enableHiding: true,
    filterFn: () => true,
  },
  {
    accessorKey: "priority",
    size: 120,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Priority" />
    ),
    cell: ({ row }) => {
      const issue = row.original
      const [currentPriority, setCurrentPriority] = React.useState(
        row.getValue("priority") as string
      )
      const [isUpdating, setIsUpdating] = React.useState(false)

      React.useEffect(() => {
        setCurrentPriority(issue.priority)
      }, [issue.priority])

      const priority = priorities.find(
        (priority) => priority.value === currentPriority
      )

      if (!priority) {
        return null
      }

      const PriorityIcon = priority.icon

      const handleChange = async (newPriority: string) => {
        if (isUpdating || newPriority === currentPriority) return

        setIsUpdating(true)
        try {
          const success = await updateIssuePriority(
            orgSlug,
            issue.display_key,
            newPriority
          )
          if (success) {
            setCurrentPriority(newPriority)
            updateIssueInCaches(issue.display_key, { priority: newPriority })
          }
        } finally {
          setIsUpdating(false)
        }
      }

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild disabled={isUpdating}>
            <button className="focus:outline-none">
              <Badge
                variant="outline"
                className="flex w-fit cursor-pointer items-center justify-start gap-2 hover:bg-muted/5"
              >
                {PriorityIcon && (
                  <PriorityIcon className="size-3.5 text-muted-foreground" />
                )}
                <span className="text-xs">{priority.label}</span>
              </Badge>
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {priorities.map((p) => {
              const Icon = p.icon
              const isActive = p.value === currentPriority
              return (
                <DropdownMenuItem
                  key={p.value}
                  onClick={() => handleChange(p.value)}
                  className="gap-2 text-xs"
                >
                  {Icon && <Icon className="h-3.5 w-3.5" />}
                  {p.label}
                  {isActive && <Check className="h-3.5 w-3.5 ml-auto" />}
                </DropdownMenuItem>
              )
            })}
          </DropdownMenuContent>
        </DropdownMenu>
      )
    },
    filterFn: (row, id, value) => {
      return (value as string[]).includes(row.getValue(id))
    },
  },
  {
    id: "actions",
    size: 40,
    cell: ({ row }) => <DataTableRowActions row={row} />,
  },
]
