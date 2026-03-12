"use client"

import * as React from "react"
import {
  flexRender,
  getCoreRowModel,
  getFacetedUniqueValues,
  useReactTable,
  type ColumnDef,
  type ColumnFiltersState,
  type SortingState,
  type VisibilityState,
} from "@tanstack/react-table"

import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

import { DataTableToolbar } from "./data-table-toolbar"
import {
  fetchMyIssues,
  type IssueSortBy,
  type FetchMyIssuesParams,
  type IssueFilterType,
} from "@/lib/my-issues"

import { type IssueFacets, type Issue } from "../data/schema"
import { DataTableSelectionOverlay } from "./data-table-selection-overlay"
import { RotateCw } from "lucide-react"

const ROW_HEIGHT = 49
const OVERSCAN = 2
const PAGE_SIZE = 50
const LOAD_MORE_THRESHOLD = 300

/** Kolon id → backend sort_by. Backend: created_at | updated_at | key_seq | title | status | priority */
const COLUMN_TO_SORT_BY: Record<string, IssueSortBy> = {
  id: "key_seq",
  title: "title",
  status: "status",
  priority: "priority",
}

function buildFetchParams(
  sorting: SortingState,
  columnFilters: ColumnFiltersState,
  cursor: string | null,
  filterType: IssueFilterType
): FetchMyIssuesParams {
  const params: FetchMyIssuesParams = {
    limit: PAGE_SIZE,
    filter_type: filterType,
    ...(cursor ? { cursor } : { offset: 0 }),
  }

  const sort = sorting[0]
  if (sort && COLUMN_TO_SORT_BY[sort.id]) {
    params.sort_by = COLUMN_TO_SORT_BY[sort.id]
    params.sort_dir = sort.desc ? "desc" : "asc"
  }

  for (const f of columnFilters) {
    const v = f.value
    if (v === undefined || v === "") continue
    if (f.id === "title") {
      params.q = typeof v === "string" ? v : String(v)
    } else if (f.id === "status") {
      params.status = Array.isArray(v) ? (v as string[]) : [v as string]
    } else if (f.id === "priority") {
      params.priority = Array.isArray(v) ? (v as string[]) : [v as string]
    }
  }

  return params
}

interface DataTableProps {
  columns: ColumnDef<Issue, unknown>[]
  filterType: IssueFilterType
}

export function DataTable({ columns, filterType }: DataTableProps) {
  const [items, setItems] = React.useState<Issue[]>([])
  const [nextCursor, setNextCursor] = React.useState<string | null>(null)
  const [total, setTotal] = React.useState(0)
  const [hasMore, setHasMore] = React.useState(true)
  const [isInitialLoading, setIsInitialLoading] = React.useState(true)
  const [isFetchingMore, setIsFetchingMore] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const [rowSelection, setRowSelection] = React.useState({})
  const [columnVisibility, setColumnVisibility] =
    React.useState<VisibilityState>({})
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  )
  const [sorting, setSorting] = React.useState<SortingState>([])
  const [scrollTop, setScrollTop] = React.useState(0)
  const [containerHeight, setContainerHeight] = React.useState(0)
  const [facets, setFacets] = React.useState<IssueFacets | null>(null)

  const scrollContainerRef = React.useRef<HTMLDivElement>(null)

  // İlk yükleme + sort/filter değişince sıfırdan yükle
  React.useEffect(() => {
    let cancelled = false

    setItems([])
    setNextCursor(null)
    setHasMore(true)

    async function loadInitial() {
      try {
        setIsInitialLoading(true)
        setError(null)

        const params = buildFetchParams(sorting, columnFilters, null, filterType)
        const res = await fetchMyIssues(params)

        if (cancelled) return

        setItems(res.items)
        setTotal(res.total)
        setNextCursor(res.next_cursor ?? null)
        setHasMore(res.has_more)
        setFacets(res.facets)
      } catch (e) {
        if (!cancelled) {
          setError(
            e instanceof Error ? e.message : "Failed to load issues."
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
  }, [sorting, columnFilters, filterType])

  // Container yüksekliği
  React.useLayoutEffect(() => {
    const container = scrollContainerRef.current
    if (!container) return

    const updateHeight = () => setContainerHeight(container.clientHeight)
    updateHeight()

    const observer = new ResizeObserver(updateHeight)
    observer.observe(container)
    return () => observer.disconnect()
  }, [])

  // Sonraki sayfa: cursor ile fetch, listeye ekle
  const loadMore = React.useCallback(async () => {
    if (isFetchingMore || !hasMore || !nextCursor) return

    try {
      setIsFetchingMore(true)

      const params = buildFetchParams(sorting, columnFilters, nextCursor, filterType)
      const res = await fetchMyIssues(params)

      setItems((prev) => [...prev, ...res.items])
      setTotal(res.total)
      setNextCursor(res.next_cursor ?? null)
      setHasMore(res.has_more)
    } catch {
      // İsteğe bağlı: setError veya sessiz bırak
    } finally {
      setIsFetchingMore(false)
    }
  }, [nextCursor, hasMore, isFetchingMore, sorting, columnFilters, filterType])

  const table = useReactTable({
    data: items,
    columns,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      columnFilters,
    },
    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onColumnVisibilityChange: setColumnVisibility,
    getCoreRowModel: getCoreRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
  })

  // Sort/filter değişince scroll başa
  React.useEffect(() => {
    scrollContainerRef.current?.scrollTo({ top: 0 })
    setScrollTop(0)
  }, [sorting, columnFilters])

  const rows = table.getRowModel().rows
  const selectedCount = table.getSelectedRowModel().rows.length
  const visibleColumnCount =
    table.getVisibleLeafColumns().length || columns.length

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

  // Scroll: pozisyonu güncelle + dibe yakınsa loadMore
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

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <DataTableToolbar table={table} facets={facets} />

      <div className="relative flex max-h-full min-h-0 flex-1 flex-col rounded-md border overflow-hidden">
        <DataTableSelectionOverlay
          selectedCount={selectedCount}
          onClearSelection={() => table.resetRowSelection()}
        />

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
                    Loading issues...
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
                      data-state={row.getIsSelected() && "selected"}
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
                    No results.
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

