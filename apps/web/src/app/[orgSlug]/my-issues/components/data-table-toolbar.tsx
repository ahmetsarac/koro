"use client"

import * as React from "react"
import { type Table } from "@tanstack/react-table"
import { LayoutGrid, List, PlusIcon, X } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"
import { DataTableViewOptions } from "./data-table-view-options"
import { useNewIssueModal } from "@/components/issues/new-issue-modal-context"

import { priorities, iconForIssueCategory } from "../data/data"
import { type IssueFacets } from "../data/schema"
import { DataTableFacetedFilter } from "./data-table-faceted-filter"

interface DataTableToolbarProps<TData> {
  table: Table<TData>
  facets?: IssueFacets | null
  view: "list" | "board"
  onViewChange: (view: "list" | "board") => void
  blockedFilter?: boolean | undefined
  onBlockedFilterChange?: (value: boolean | undefined) => void
}

export function DataTableToolbar<TData>({
  table,
  facets = null,
  view,
  onViewChange,
  blockedFilter,
  onBlockedFilterChange,
}: DataTableToolbarProps<TData>) {
  const isFiltered =
    table.getState().columnFilters.length > 0 || blockedFilter !== undefined
  const newIssueModal = useNewIssueModal()

  const statusFilterOptions = React.useMemo(() => {
    if (!facets?.status?.length) return []
    return facets.status.map((s) => ({
      value: s.workflow_status_id,
      label: s.name,
      icon: iconForIssueCategory(s.category),
    }))
  }, [facets])

  const statusFacetCounts = React.useMemo(() => {
    if (!facets?.status?.length) return undefined
    return Object.fromEntries(
      facets.status.map((s) => [s.workflow_status_id, s.count])
    )
  }, [facets])

  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 flex-wrap items-center gap-2">
        <Input
          placeholder="Filter issues..."
          value={(table.getColumn("title")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("title")?.setFilterValue(event.target.value)
          }
          className="h-8 w-[150px] lg:w-[250px]"
        />
        {table.getColumn("status") && statusFilterOptions.length > 0 && (
          <DataTableFacetedFilter
            column={table.getColumn("status")}
            title="Status"
            options={statusFilterOptions}
            facetCounts={statusFacetCounts}
          />
        )}
        {onBlockedFilterChange && (
          <div className="inline-flex items-center rounded-md border bg-background p-0.5">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className={cn(
                "h-7 px-2 text-xs",
                blockedFilter === undefined &&
                  "bg-muted text-foreground hover:bg-muted"
              )}
              onClick={() => onBlockedFilterChange(undefined)}
            >
              All
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className={cn(
                "h-7 px-2 text-xs",
                blockedFilter === true &&
                  "bg-muted text-foreground hover:bg-muted"
              )}
              onClick={() => onBlockedFilterChange(true)}
            >
              Blocked
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className={cn(
                "h-7 px-2 text-xs",
                blockedFilter === false &&
                  "bg-muted text-foreground hover:bg-muted"
              )}
              onClick={() => onBlockedFilterChange(false)}
            >
              Not blocked
            </Button>
          </div>
        )}
        {table.getColumn("priority") && (
          <DataTableFacetedFilter
            column={table.getColumn("priority")}
            title="Priority"
            options={priorities}
            facetCounts={facets?.priority}
          />
        )}
        {isFiltered && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              table.resetColumnFilters()
              onBlockedFilterChange?.(undefined)
            }}
          >
            Reset
            <X />
          </Button>
        )}
      </div>
      <div className="flex items-center gap-2">
        <DataTableViewOptions table={table} />
        <div className="inline-flex items-center rounded-md border bg-background p-0.5">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className={cn(
              "px-2",
              view === "list" && "bg-muted text-foreground hover:bg-muted"
            )}
            onClick={() => onViewChange("list")}
          >
            <List />
            List
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className={cn(
              "px-2",
              view === "board" && "bg-muted text-foreground hover:bg-muted"
            )}
            onClick={() => onViewChange("board")}
          >
            <LayoutGrid />
            Board
          </Button>
        </div>
        <Button
          data-icon="inline-start"
          type="button"
          onClick={() => newIssueModal?.openNewIssueModal()}
        >
          <PlusIcon />
          Create Issue
        </Button>
      </div>
    </div>
  )
}
