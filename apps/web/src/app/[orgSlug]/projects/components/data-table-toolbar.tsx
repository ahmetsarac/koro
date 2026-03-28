"use client"

import { type Table } from "@tanstack/react-table"
import { PlusIcon, X } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

interface DataTableToolbarProps<TData> {
  table: Table<TData>
  searchValue: string
  onSearchChange: (value: string) => void
  onNewProject: () => void
}

export function DataTableToolbar<TData>({
  searchValue,
  onSearchChange,
  onNewProject,
}: DataTableToolbarProps<TData>) {
  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 items-center gap-2">
        <Input
          placeholder="Search projects..."
          value={searchValue}
          onChange={(event) => onSearchChange(event.target.value)}
          className="h-8 w-[150px] lg:w-[250px]"
        />
        {searchValue && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onSearchChange("")}
          >
            Reset
            <X />
          </Button>
        )}
      </div>
      <div className="flex items-center gap-2">
        <Button
          type="button"
          data-icon="inline-start"
          size="lg"
          onClick={onNewProject}
        >
          <PlusIcon />
          New Project
        </Button>
      </div>
    </div>
  )
}
