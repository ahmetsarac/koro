"use client"

import { type Table } from "@tanstack/react-table"
import { PlusIcon, X } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { DataTableViewOptions } from "./data-table-view-options"

import { labels, priorities, statuses } from "../data/data"
import { type DemoTaskFacets } from "../data/schema"
import { DataTableFacetedFilter } from "./data-table-faceted-filter"

interface DataTableToolbarProps<TData> {
  table: Table<TData>
  /** Server-side facet counts (from full result set); when set, filter dropdowns show these instead of client-loaded counts */
  facets?: DemoTaskFacets | null
}

export function DataTableToolbar<TData>({
  table,
  facets = null,
}: DataTableToolbarProps<TData>) {
  const isFiltered = table.getState().columnFilters.length > 0

  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 items-center gap-2">
        <Input
          placeholder="Filter tasks..."
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
        {table.getColumn("label") && (
          <DataTableFacetedFilter
            column={table.getColumn("label")}
            title="Label"
            options={labels}
            facetCounts={facets?.label}
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
        <Button data-icon="inline-start" size="lg"><PlusIcon />Create Issue</Button>
      </div>
    </div>
  )
}
