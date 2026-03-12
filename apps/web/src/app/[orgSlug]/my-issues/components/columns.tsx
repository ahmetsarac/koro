"use client"

import * as React from "react"
import Link from "next/link"
import { type ColumnDef } from "@tanstack/react-table"

import { Checkbox } from "@/components/ui/checkbox"
import { Badge } from "@/components/ui/badge"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Check } from "lucide-react"

import { priorities, statuses } from "../data/data"
import { type Issue } from "../data/schema"
import { DataTableColumnHeader } from "./data-table-column-header"
import { DataTableRowActions } from "./data-table-row-actions"
import { updateIssueInCaches } from "@/lib/cache/issues-cache"

async function updateIssueStatus(issueId: string, status: string): Promise<boolean> {
  const response = await fetch(`/api/issues/${issueId}/status`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ status }),
  })

  return response.ok
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
    accessorKey: "status",
    size: 120,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Status" />
    ),
    cell: ({ row }) => {
      const issue = row.original
      const [currentStatus, setCurrentStatus] = React.useState(
        row.getValue("status") as string
      )
      const [isUpdating, setIsUpdating] = React.useState(false)

      const status = statuses.find(
        (status) => status.value === currentStatus
      )

      if (!status) {
        return null
      }

      const StatusIcon = status.icon

      const handleChange = async (newStatus: string) => {
        if (isUpdating || newStatus === currentStatus) return

        setIsUpdating(true)
        try {
          const success = await updateIssueStatus(issue.id, newStatus)
          if (success) {
            setCurrentStatus(newStatus)
            updateIssueInCaches(issue.display_key, { status: newStatus })
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
                className="flex w-[110px] items-center gap-2 justify-start cursor-pointer hover:bg-muted/60"
              >
                {StatusIcon && (
                  <StatusIcon className="size-3.5 text-muted-foreground" />
                )}
                <span className="text-xs">{status.label}</span>
              </Badge>
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {statuses.map((s) => {
              const Icon = s.icon
              const isActive = s.value === currentStatus
              return (
                <DropdownMenuItem
                  key={s.value}
                  onClick={() => handleChange(s.value)}
                  className="gap-2 text-xs"
                >
                  {Icon && <Icon className="h-3.5 w-3.5" />}
                  {s.label}
                  {isActive && <Check className="h-3.5 w-3.5 ml-auto" />}
                </DropdownMenuItem>
              )
            })}
          </DropdownMenuContent>
        </DropdownMenu>
      )
    },
    filterFn: (row, id, value) => {
      return value.includes(row.getValue(id))
    },
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
                className="flex w-[110px] items-center gap-2 justify-start cursor-pointer hover:bg-muted/60"
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
      return value.includes(row.getValue(id))
    },
  },
  {
    id: "actions",
    size: 40,
    cell: ({ row }) => <DataTableRowActions row={row} />,
  },
]
