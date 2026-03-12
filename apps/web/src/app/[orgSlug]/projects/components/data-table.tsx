"use client"

import * as React from "react"
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table"

import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

import { DataTableToolbar } from "./data-table-toolbar"
import { fetchMyProjects, type FetchMyProjectsParams } from "@/lib/my-projects"
import { type Project } from "../data/schema"
import { RotateCw } from "lucide-react"

const ROW_HEIGHT = 49
const OVERSCAN = 2
const PAGE_SIZE = 50
const LOAD_MORE_THRESHOLD = 300

function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = React.useState<T>(value)

  React.useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value)
    }, delay)

    return () => {
      clearTimeout(handler)
    }
  }, [value, delay])

  return debouncedValue
}

interface DataTableProps {
  columns: ColumnDef<Project, unknown>[]
}

export function DataTable({ columns }: DataTableProps) {
  const [items, setItems] = React.useState<Project[]>([])
  const [total, setTotal] = React.useState(0)
  const [hasMore, setHasMore] = React.useState(true)
  const [isInitialLoading, setIsInitialLoading] = React.useState(true)
  const [isFetchingMore, setIsFetchingMore] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const [searchValue, setSearchValue] = React.useState("")
  const debouncedSearch = useDebounce(searchValue, 300)

  const [scrollTop, setScrollTop] = React.useState(0)
  const [containerHeight, setContainerHeight] = React.useState(0)

  const scrollContainerRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    let cancelled = false

    setItems([])
    setHasMore(true)

    async function loadInitial() {
      try {
        setIsInitialLoading(true)
        setError(null)

        const params: FetchMyProjectsParams = {
          limit: PAGE_SIZE,
          offset: 0,
          ...(debouncedSearch && { q: debouncedSearch }),
        }
        const res = await fetchMyProjects(params)

        if (cancelled) return

        setItems(res.items)
        setTotal(res.total)
        setHasMore(res.has_more)
      } catch (e) {
        if (!cancelled) {
          setError(
            e instanceof Error ? e.message : "Failed to load projects."
          )
        }
      } finally {
        if (!cancelled) setIsInitialLoading(false)
      }
    }

    loadInitial()
    return () => {
      cancelled = true
    }
  }, [debouncedSearch])

  React.useLayoutEffect(() => {
    const container = scrollContainerRef.current
    if (!container) return

    const updateHeight = () => setContainerHeight(container.clientHeight)
    updateHeight()

    const observer = new ResizeObserver(updateHeight)
    observer.observe(container)
    return () => observer.disconnect()
  }, [])

  const loadMore = React.useCallback(async () => {
    if (isFetchingMore || !hasMore) return

    try {
      setIsFetchingMore(true)

      const params: FetchMyProjectsParams = {
        limit: PAGE_SIZE,
        offset: items.length,
        ...(debouncedSearch && { q: debouncedSearch }),
      }
      const res = await fetchMyProjects(params)

      setItems((prev) => [...prev, ...res.items])
      setTotal(res.total)
      setHasMore(res.has_more)
    } catch {
      // Silent fail for load more
    } finally {
      setIsFetchingMore(false)
    }
  }, [items.length, hasMore, isFetchingMore, debouncedSearch])

  const table = useReactTable({
    data: items,
    columns,
    getCoreRowModel: getCoreRowModel(),
  })

  React.useEffect(() => {
    scrollContainerRef.current?.scrollTo({ top: 0 })
    setScrollTop(0)
  }, [debouncedSearch])

  const rows = table.getRowModel().rows
  const visibleColumnCount = table.getVisibleLeafColumns().length || columns.length

  const visibleCount =
    containerHeight > 0 ? Math.ceil(containerHeight / ROW_HEIGHT) : 12

  const startIndex = Math.max(
    0,
    Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN
  )
  const endIndex = Math.min(
    rows.length,
    startIndex + visibleCount + OVERSCAN * 2
  )

  const visibleRows = rows.slice(startIndex, endIndex)
  const topSpacerHeight = startIndex * ROW_HEIGHT
  const bottomSpacerHeight = Math.max(0, (rows.length - endIndex) * ROW_HEIGHT)

  const handleScroll = React.useCallback(
    (event: React.UIEvent<HTMLDivElement>) => {
      const el = event.currentTarget
      setScrollTop(el.scrollTop)

      const distanceToBottom =
        el.scrollHeight - el.scrollTop - el.clientHeight

      if (
        distanceToBottom < LOAD_MORE_THRESHOLD &&
        hasMore &&
        !isFetchingMore
      ) {
        void loadMore()
      }
    },
    [hasMore, isFetchingMore, loadMore]
  )

  const colGroup = (
    <colgroup>
      {table.getVisibleLeafColumns().map((column) => (
        <col key={column.id} style={{ width: column.getSize() }} />
      ))}
    </colgroup>
  )

  const HEADER_HEIGHT = 41
  const contentHeight = HEADER_HEIGHT + rows.length * ROW_HEIGHT

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <DataTableToolbar
        table={table}
        searchValue={searchValue}
        onSearchChange={setSearchValue}
      />

      <div
        className="relative flex flex-col rounded-md border overflow-hidden"
        style={{ height: `min(${contentHeight}px, 100%)` }}
      >
        <table className="w-full text-xs" style={{ tableLayout: "fixed" }}>
          {colGroup}
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id} className="border-none">
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    colSpan={header.colSpan}
                    className="bg-background shadow-[inset_0_-1px_0_0_var(--border)]"
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
        </table>

        <div
          ref={scrollContainerRef}
          className="relative flex-1 overflow-y-auto min-h-0"
          onScroll={handleScroll}
        >
          <table className="w-full text-xs" style={{ tableLayout: "fixed" }}>
            {colGroup}
            <TableBody>
              {isInitialLoading ? (
                <TableRow>
                  <TableCell
                    colSpan={visibleColumnCount}
                    className="h-24 text-center"
                  >
                    Loading projects...
                  </TableCell>
                </TableRow>
              ) : error ? (
                <TableRow>
                  <TableCell
                    colSpan={visibleColumnCount}
                    className="h-24 text-center text-destructive"
                  >
                    {error}
                  </TableCell>
                </TableRow>
              ) : rows.length ? (
                <>
                  {topSpacerHeight > 0 && (
                    <TableRow
                      aria-hidden="true"
                      className="hover:bg-transparent"
                    >
                      <TableCell
                        colSpan={visibleColumnCount}
                        className="p-0"
                        style={{ height: topSpacerHeight }}
                      />
                    </TableRow>
                  )}

                  {visibleRows.map((row) => (
                    <TableRow
                      key={row.id}
                      style={{ height: ROW_HEIGHT }}
                    >
                      {row.getVisibleCells().map((cell) => (
                        <TableCell key={cell.id}>
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext()
                          )}
                        </TableCell>
                      ))}
                    </TableRow>
                  ))}

                  {bottomSpacerHeight > 0 && (
                    <TableRow
                      aria-hidden="true"
                      className="hover:bg-transparent"
                    >
                      <TableCell
                        colSpan={visibleColumnCount}
                        className="p-0"
                        style={{ height: bottomSpacerHeight }}
                      />
                    </TableRow>
                  )}
                </>
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={visibleColumnCount}
                    className="h-24 text-center"
                  >
                    No projects found.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </table>
        </div>
      </div>

      <div className="relative flex items-center justify-center px-2 py-1 text-sm text-muted-foreground">
        <span>
          {total > 0 ? `${items.length} / ${total}` : "—"}
        </span>
        
        {isFetchingMore && (
          <span className="absolute right-2 flex items-center text-xs">
            <RotateCw className="w-3 h-3 mr-1.5 animate-spin" />
            Loading…
          </span>
        )}
      </div>
    </div>
  )
}
