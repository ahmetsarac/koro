"use client"

import * as React from "react"
import { type Table } from "@tanstack/react-table"
import { Ban, LayoutGrid, Link2, List, PlusIcon, X } from "lucide-react"

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
  boardHiddenColumns?: { id: string; label: string; issueCount: number }[]
  onUnhideBoardColumn?: (columnId: string) => void
  hideZeroCountBoardColumns?: boolean
  onHideZeroCountBoardColumnsChange?: (value: boolean) => void
}

export function DataTableToolbar<TData>({
  table,
  facets = null,
  view,
  onViewChange,
  boardHiddenColumns = [],
  onUnhideBoardColumn,
  hideZeroCountBoardColumns = false,
  onHideZeroCountBoardColumnsChange,
}: DataTableToolbarProps<TData>) {
  const isFiltered = table.getState().columnFilters.length > 0
  const newIssueModal = useNewIssueModal()

  const statusFilterOptions = React.useMemo(() => {
    if (!facets?.status?.length) return []
    return facets.status.map((s) => ({
      value: s.slug,
      label: s.name,
      icon: iconForIssueCategory(s.category),
    }))
  }, [facets])

  const statusFacetCounts = React.useMemo(() => {
    if (!facets?.status?.length) return undefined
    return Object.fromEntries(facets.status.map((s) => [s.slug, s.count]))
  }, [facets])

  const relationsFilterOptions = React.useMemo(
    () => [
      { value: "blocked", label: "Blocked", icon: Ban },
      { value: "blocking", label: "Blocking", icon: Link2 },
    ],
    []
  )

  const relationsFacetCounts = React.useMemo(() => {
    if (!facets?.relations) return undefined
    return {
      blocked: facets.relations.blocked,
      blocking: facets.relations.blocking,
    }
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
        {table.getColumn("relations") && (
          <DataTableFacetedFilter
            column={table.getColumn("relations")}
            title="Relations"
            options={relationsFilterOptions}
            facetCounts={relationsFacetCounts}
          />
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
            onClick={() => table.resetColumnFilters()}
          >
            Reset
            <X />
          </Button>
        )}
      </div>
      <div className="flex items-center gap-2">
        <DataTableViewOptions
          table={table}
          view={view}
          boardHiddenColumns={boardHiddenColumns}
          onUnhideBoardColumn={onUnhideBoardColumn}
          hideZeroCountBoardColumns={hideZeroCountBoardColumns}
          onHideZeroCountBoardColumnsChange={
            onHideZeroCountBoardColumnsChange
          }
        />
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
