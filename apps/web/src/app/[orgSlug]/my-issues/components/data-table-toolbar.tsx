"use client"

import { type Table } from "@tanstack/react-table"
import { LayoutGrid, List, PlusIcon, X } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"
import { DataTableViewOptions } from "./data-table-view-options"
import { useNewIssueModal } from "@/components/issues/new-issue-modal-context"

import { priorities, statuses } from "../data/data"
import { type IssueFacets } from "../data/schema"
import { DataTableFacetedFilter } from "./data-table-faceted-filter"

interface DataTableToolbarProps<TData> {
  table: Table<TData>
  facets?: IssueFacets | null
  view: "list" | "board"
  onViewChange: (view: "list" | "board") => void
}

export function DataTableToolbar<TData>({
  table,
  facets = null,
  view,
  onViewChange,
}: DataTableToolbarProps<TData>) {
  const isFiltered = table.getState().columnFilters.length > 0
  const newIssueModal = useNewIssueModal()

  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 items-center gap-2">
        <Input
          placeholder="Filter issues..."
          value={(table.getColumn("title")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("title")?.setFilterValue(event.target.value)
          }
          className="h-8 w-[150px] lg:w-[250px]"
        />
        {table.getColumn("status") && (
          <DataTableFacetedFilter
            column={table.getColumn("status")}
            title="Status"
            options={statuses}
            facetCounts={facets?.status}
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
          size="lg"
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
