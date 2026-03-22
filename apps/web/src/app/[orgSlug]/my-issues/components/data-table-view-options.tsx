"use client"

import * as React from "react"
import { type Table } from "@tanstack/react-table"
import { Eye, Settings2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import { Separator } from "@/components/ui/separator"
import { cn } from "@/lib/utils"

const viewTriggerClassName = "hidden h-8 shrink-0 lg:flex"

export interface DataTableViewOptionsProps<TData> {
  table: Table<TData>
  view: "list" | "board"
  boardHiddenColumns?: { id: string; label: string; issueCount: number }[]
  onUnhideBoardColumn?: (columnId: string) => void
  hideZeroCountBoardColumns?: boolean
  onHideZeroCountBoardColumnsChange?: (value: boolean) => void
}

export function DataTableViewOptions<TData>({
  table,
  view,
  boardHiddenColumns = [],
  onUnhideBoardColumn,
  hideZeroCountBoardColumns = false,
  onHideZeroCountBoardColumnsChange,
}: DataTableViewOptionsProps<TData>) {
  const hideZeroCheckboxId = React.useId()

  const triggerButton = (
    <Button variant="outline" size="sm" className={viewTriggerClassName}>
      <Settings2 />
      View
    </Button>
  )

  if (view === "board") {
    if (!onHideZeroCountBoardColumnsChange) {
      return null
    }

    return (
      <Popover>
        <PopoverTrigger asChild>{triggerButton}</PopoverTrigger>
        <PopoverContent
          className="max-h-none w-72 overflow-visible p-0"
          align="end"
        >
          <div className="space-y-3 overflow-visible p-3">
            <div className="flex items-start gap-2.5">
              <Checkbox
                id={hideZeroCheckboxId}
                checked={hideZeroCountBoardColumns}
                onCheckedChange={(checked) =>
                  onHideZeroCountBoardColumnsChange(checked === true)
                }
              />
              <label
                htmlFor={hideZeroCheckboxId}
                className="cursor-pointer text-xs leading-snug font-medium"
              >
                Hide columns with 0 issues
              </label>
            </div>
            {boardHiddenColumns.length > 0 && onUnhideBoardColumn ? (
              <>
                <Separator />
                <div>
                  <div className="mb-1.5 text-xs font-medium text-muted-foreground">
                    Manually hidden
                  </div>
                  <ul
                    className={cn(
                      "space-y-1",
                      boardHiddenColumns.length > 8
                        ? "max-h-52 overflow-y-auto overscroll-y-contain"
                        : "overflow-visible"
                    )}
                  >
                    {boardHiddenColumns.map((col) => (
                      <li key={col.id}>
                        <div className="flex items-center gap-1 rounded-sm py-0.5 pr-1 hover:bg-muted/60">
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7 shrink-0 text-muted-foreground hover:text-foreground"
                            aria-label={`Show column ${col.label}`}
                            title="Show column"
                            onClick={() => onUnhideBoardColumn(col.id)}
                          >
                            <Eye className="h-4 w-4" />
                          </Button>
                          <div className="flex min-w-0 flex-1 items-center gap-2 text-xs">
                            <span className="truncate font-medium">
                              {col.label}
                            </span>
                            <span className="ml-auto shrink-0 tabular-nums text-muted-foreground">
                              {col.issueCount}
                            </span>
                          </div>
                        </div>
                      </li>
                    ))}
                  </ul>
                </div>
              </>
            ) : null}
          </div>
        </PopoverContent>
      </Popover>
    )
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>{triggerButton}</DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-[150px]">
        <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {table
          .getAllColumns()
          .filter(
            (column) =>
              typeof column.accessorFn !== "undefined" && column.getCanHide()
          )
          .map((column) => {
            return (
              <DropdownMenuCheckboxItem
                key={column.id}
                className="capitalize"
                checked={column.getIsVisible()}
                onCheckedChange={(value) => column.toggleVisibility(!!value)}
              >
                {column.id}
              </DropdownMenuCheckboxItem>
            )
          })}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
