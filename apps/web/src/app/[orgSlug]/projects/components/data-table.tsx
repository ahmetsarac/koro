"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
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

import { NewProjectModal } from "@/components/projects/new-project-modal"
import { DataTableToolbar } from "./data-table-toolbar"
import { fetchMyProjects, type FetchMyProjectsParams } from "@/lib/my-projects"
import { type Project } from "../data/schema"
import { cn } from "@/lib/utils"
import { RotateCw } from "lucide-react"

const ROW_HEIGHT = 49
const OVERSCAN = 2
const PAGE_SIZE = 50
const LOAD_MORE_THRESHOLD = 300
/** Extra outer height only when overflow is tiny (border/subpixel), not real scrolling */
const SCROLL_ARTIFACT_MAX_PX = 8

function projectsEmptyCopy(searchQuery: string): {
  title: string
  description: string
} {
  if (searchQuery.trim()) {
    return {
      title: "No projects match your search",
      description:
        "Try another name or project key, or clear the search box.",
    }
  }
  return {
    title: "No projects yet",
    description:
      "Create a project to start tracking issues in this organization.",
  }
}

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
  orgSlug: string
}

export function DataTable({ columns, orgSlug }: DataTableProps) {
  const router = useRouter()
  const [newProjectOpen, setNewProjectOpen] = React.useState(false)
  const [listRefreshKey, setListRefreshKey] = React.useState(0)

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
  const [bodyScrollbarWidth, setBodyScrollbarWidth] = React.useState(0)

  const scrollContainerRef = React.useRef<HTMLDivElement>(null)
  const headerShellRef = React.useRef<HTMLDivElement>(null)
  const [headerShellHeight, setHeaderShellHeight] = React.useState(41)
  const [scrollArtifactFudge, setScrollArtifactFudge] = React.useState(0)

  React.useEffect(() => {
    let cancelled = false

    setItems([])
    setHasMore(true)

    async function loadInitial() {
      try {
        setIsInitialLoading(true)
        setError(null)

        const params: FetchMyProjectsParams = {
          orgSlug,
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
  }, [debouncedSearch, listRefreshKey, orgSlug])

  const measureBodyScrollbarWidth = React.useCallback(() => {
    const el = scrollContainerRef.current
    if (!el) {
      setBodyScrollbarWidth(0)
      return
    }
    setBodyScrollbarWidth(el.offsetWidth - el.clientWidth)
  }, [])

  React.useLayoutEffect(() => {
    const container = scrollContainerRef.current
    if (!container) return

    const updateHeight = () => setContainerHeight(container.clientHeight)
    updateHeight()
    measureBodyScrollbarWidth()

    const observer = new ResizeObserver(() => {
      updateHeight()
      measureBodyScrollbarWidth()
    })
    observer.observe(container)
    return () => observer.disconnect()
  }, [measureBodyScrollbarWidth])

  const loadMore = React.useCallback(async () => {
    if (isFetchingMore || !hasMore) return

    try {
      setIsFetchingMore(true)

      const params: FetchMyProjectsParams = {
        orgSlug,
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
  }, [orgSlug, items.length, hasMore, isFetchingMore, debouncedSearch])

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

  const contentHeight =
    headerShellHeight + rows.length * ROW_HEIGHT + scrollArtifactFudge

  const showProjectsEmpty =
    !isInitialLoading && !error && rows.length === 0
  const projectsEmpty = projectsEmptyCopy(debouncedSearch)

  React.useLayoutEffect(() => {
    const shell = headerShellRef.current
    if (!shell) return

    const measure = () => {
      const h = shell.getBoundingClientRect().height
      setHeaderShellHeight(Math.ceil(h))
    }
    measure()
    const ro = new ResizeObserver(measure)
    ro.observe(shell)
    return () => ro.disconnect()
  }, [bodyScrollbarWidth, isInitialLoading, error, rows.length, showProjectsEmpty])

  React.useLayoutEffect(() => {
    measureBodyScrollbarWidth()
  }, [
    measureBodyScrollbarWidth,
    rows.length,
    containerHeight,
    isInitialLoading,
    error,
    showProjectsEmpty,
  ])

  React.useLayoutEffect(() => {
    const el = scrollContainerRef.current
    if (!el || rows.length === 0 || isInitialLoading || error) {
      setScrollArtifactFudge(0)
      return
    }
    const overflow = Math.ceil(
      Math.max(0, el.scrollHeight - el.clientHeight - Number.EPSILON)
    )
    if (overflow > SCROLL_ARTIFACT_MAX_PX) {
      setScrollArtifactFudge(0)
      return
    }
    if (overflow > 0) {
      setScrollArtifactFudge((prev) => (overflow > prev ? overflow : prev))
    }
  }, [
    rows.length,
    containerHeight,
    headerShellHeight,
    isInitialLoading,
    error,
    showProjectsEmpty,
    bodyScrollbarWidth,
  ])

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <NewProjectModal
        orgSlug={orgSlug}
        open={newProjectOpen}
        onOpenChange={setNewProjectOpen}
        onSuccess={() => {
          setListRefreshKey((k) => k + 1)
          router.refresh()
        }}
      />
      <DataTableToolbar
        table={table}
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        onNewProject={() => setNewProjectOpen(true)}
      />

      <div
        className={`relative flex flex-col overflow-hidden rounded-md border${showProjectsEmpty ? " min-h-[140px]" : ""}`}
        style={{ height: `min(${contentHeight}px, 100%)` }}
      >
        <div
          ref={headerShellRef}
          className="min-w-0 shrink-0"
          style={{ paddingInlineEnd: bodyScrollbarWidth }}
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
        </div>

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
                      className={cn(
                        bottomSpacerHeight === 0 &&
                          row.index === rows.length - 1 &&
                          "border-b-0"
                      )}
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
                      className="hover:bg-transparent border-b-0"
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
                <TableRow className="hover:bg-transparent">
                  <TableCell
                    colSpan={visibleColumnCount}
                    className="h-auto min-h-[120px] align-middle px-6 py-6"
                  >
                    <div className="flex flex-col items-center justify-center gap-1.5 text-center">
                      <p className="text-sm font-medium text-foreground">
                        {projectsEmpty.title}
                      </p>
                      <p className="max-w-sm text-xs text-muted-foreground">
                        {projectsEmpty.description}
                      </p>
                    </div>
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
