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

interface WorkflowOption {
  id: string
  category: string
  name: string
  slug: string
  position: number
  is_default: boolean
}

const workflowOptionsCache = new Map<string, WorkflowOption[]>()
const workflowOptionsInflight = new Map<string, Promise<WorkflowOption[]>>()

function workflowCacheKey(orgSlug: string, projectKey: string) {
  return `${orgSlug}\0${projectKey}`
}

function projectKeyFromDisplayKey(displayKey: string): string {
  const i = displayKey.lastIndexOf("-")
  return i >= 0 ? displayKey.slice(0, i) : displayKey
}

async function getWorkflowStatusOptions(
  orgSlug: string,
  projectKey: string
): Promise<WorkflowOption[]> {
  const k = workflowCacheKey(orgSlug, projectKey)
  const hit = workflowOptionsCache.get(k)
  if (hit) return hit
  const inflight = workflowOptionsInflight.get(k)
  if (inflight) return inflight

  const p = fetch(
    `/api/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses`,
    { cache: "no-store", credentials: "same-origin" }
  )
    .then(async (response) => {
      const data = (response.ok
        ? await response.json()
        : { groups: [] }) as {
        groups?: { category: string; statuses: WorkflowOption[] }[]
      }
      const flat: WorkflowOption[] = []
      for (const g of data.groups ?? []) {
        for (const s of g.statuses ?? []) {
          flat.push({ ...s, category: s.category || g.category })
        }
      }
      flat.sort(
        (a, b) =>
          a.category.localeCompare(b.category) || a.position - b.position
      )
      workflowOptionsCache.set(k, flat)
      return flat
    })
    .finally(() => {
      workflowOptionsInflight.delete(k)
    })

  workflowOptionsInflight.set(k, p)
  return p
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
        href={`/${orgSlug}/issue/${row.original.display_key}`}
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
        href={`/${orgSlug}/issue/${row.original.display_key}`}
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
              href={`/${orgSlug}/issue/${issue.display_key}`}
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
      return v.includes(row.original.workflow_status_id)
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
