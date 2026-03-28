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
import { IssueKanbanBoard } from "@/components/issues/issue-kanban-board"
import { useNewIssueModal } from "@/components/issues/new-issue-modal-context"

import { DataTableToolbar } from "./data-table-toolbar"
import {
  fetchMyIssues,
  type IssueSortBy,
  type FetchMyIssuesParams,
  type IssueFilterType,
} from "@/lib/my-issues"

import { type IssueFacets, type Issue } from "../data/schema"
import { iconForIssueCategory } from "../data/data"
import { useMyIssuesInitialView } from "./my-issues-view-context"
import { DataTableSelectionOverlay } from "./data-table-selection-overlay"
import { MY_ISSUES_VIEW_COOKIE } from "../constants"
import { RotateCw } from "lucide-react"
import {
  myIssuesCache,
  MY_ISSUES_VIEW_STORAGE_KEY,
  type MyIssuesScrollState,
  updateIssueInCaches,
} from "@/lib/cache/issues-cache"
import { issueDetailHref } from "@/lib/issue-nav"

const ROW_HEIGHT = 49
const OVERSCAN = 2
const PAGE_SIZE = 50
const LOAD_MORE_THRESHOLD = 300

interface ScrollState {
  scrollTop: number
  items: Issue[]
  nextCursor: string | null
  total: number
  hasMore: boolean
  facets: IssueFacets | null
}

function saveScrollState(filterType: IssueFilterType, state: ScrollState): void {
  myIssuesCache.set(filterType, state as MyIssuesScrollState)
}

function loadScrollState(filterType: IssueFilterType): ScrollState | null {
  return myIssuesCache.get(filterType) as ScrollState | null ?? null
}

function clearScrollState(filterType: IssueFilterType): void {
  myIssuesCache.delete(filterType)
}

function loadViewPreference(): "list" | "board" {
  if (typeof window === "undefined") return "list"
  const saved = window.localStorage.getItem(MY_ISSUES_VIEW_STORAGE_KEY)
  return saved === "board" ? "board" : "list"
}

function saveViewPreference(view: "list" | "board"): void {
  if (typeof window === "undefined") return
  window.localStorage.setItem(MY_ISSUES_VIEW_STORAGE_KEY, view)
  document.cookie = `${MY_ISSUES_VIEW_COOKIE}=${view}; path=/; max-age=31536000; SameSite=Lax`
}

/** Kolon id → backend sort_by. Backend: created_at | updated_at | key_seq | title | status | priority */
const COLUMN_TO_SORT_BY: Record<string, IssueSortBy> = {
  id: "key_seq",
  title: "title",
  status: "status",
  priority: "priority",
}

function buildFetchParams(
  orgSlug: string,
  sorting: SortingState,
  columnFilters: ColumnFiltersState,
  cursor: string | null,
  filterType: IssueFilterType
): FetchMyIssuesParams {
  const params: FetchMyIssuesParams = {
    orgSlug,
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
    } else if (f.id === "relations") {
      params.relations = Array.isArray(v) ? (v as string[]) : [v as string]
    }
  }

  return params
}

function myIssuesEmptyCopy(
  filterType: IssueFilterType,
  hasActiveFilters: boolean
): { title: string; description: string } {
  if (hasActiveFilters) {
    return {
      title: "No matching issues",
      description:
        "Try adjusting search or column filters to see more results.",
    }
  }
  if (filterType === "created") {
    return {
      title: "No issues created by you",
      description:
        "Issues you create in this organization will appear here.",
    }
  }
  return {
    title: "Nothing assigned to you",
    description:
      "When someone assigns you an issue in this organization, it will show up here.",
  }
}

const WORKFLOW_CATEGORY_ORDER = [
  "backlog",
  "unstarted",
  "started",
  "completed",
  "canceled",
] as const

function categoryRank(category: string): number {
  const i = WORKFLOW_CATEGORY_ORDER.indexOf(
    category as (typeof WORKFLOW_CATEGORY_ORDER)[number]
  )
  return i === -1 ? 99 : i
}

async function updateIssueBoardPosition(
  issueId: string,
  workflow_status_id: string,
  position: number
): Promise<boolean> {
  const response = await fetch(`/api/issues/${issueId}/board-position`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ workflow_status_id, position }),
  })

  return response.ok
}

interface DataTableProps {
  orgSlug: string
  columns: ColumnDef<Issue, unknown>[]
  filterType: IssueFilterType
}

export function DataTable({ orgSlug, columns, filterType }: DataTableProps) {
  const newIssueModal = useNewIssueModal()
  const initialView = useMyIssuesInitialView()
  const [items, setItems] = React.useState<Issue[]>([])
  const [nextCursor, setNextCursor] = React.useState<string | null>(null)
  const [total, setTotal] = React.useState(0)
  const [hasMore, setHasMore] = React.useState(true)
  const [isInitialLoading, setIsInitialLoading] = React.useState(true)
  const [isFetchingMore, setIsFetchingMore] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const [rowSelection, setRowSelection] = React.useState({})
  const [columnVisibility, setColumnVisibility] =
    React.useState<VisibilityState>({ relations: false })
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  )
  const [sorting, setSorting] = React.useState<SortingState>([])
  const [scrollTop, setScrollTop] = React.useState(0)
  const [containerHeight, setContainerHeight] = React.useState(0)
  const [listBodyOverflows, setListBodyOverflows] = React.useState(false)
  const [facets, setFacets] = React.useState<IssueFacets | null>(null)
  const boardHiddenStorageKey = `koro_my_issues_hidden_board_cols_${orgSlug}_${filterType}`
  const hideZeroBoardStorageKey = `koro_my_issues_hide_zero_board_cols_${orgSlug}_${filterType}`
  const [hiddenBoardColumnIds, setHiddenBoardColumnIds] = React.useState<
    Set<string>
  >(() => new Set())
  const [hideZeroCountBoardColumns, setHideZeroCountBoardColumns] =
    React.useState(false)

  React.useLayoutEffect(() => {
    try {
      const raw = sessionStorage.getItem(boardHiddenStorageKey)
      setHiddenBoardColumnIds(
        raw ? new Set(JSON.parse(raw) as string[]) : new Set()
      )
    } catch {
      setHiddenBoardColumnIds(new Set())
    }
  }, [boardHiddenStorageKey])

  React.useLayoutEffect(() => {
    try {
      const raw = sessionStorage.getItem(hideZeroBoardStorageKey)
      setHideZeroCountBoardColumns(raw === "1" || raw === "true")
    } catch {
      setHideZeroCountBoardColumns(false)
    }
  }, [hideZeroBoardStorageKey])

  const hideBoardColumn = React.useCallback(
    (columnId: string) => {
      setHiddenBoardColumnIds((prev) => {
        const next = new Set(prev)
        next.add(columnId)
        try {
          sessionStorage.setItem(
            boardHiddenStorageKey,
            JSON.stringify([...next])
          )
        } catch {
          /* ignore */
        }
        return next
      })
    },
    [boardHiddenStorageKey]
  )

  const unhideBoardColumn = React.useCallback(
    (columnId: string) => {
      setHiddenBoardColumnIds((prev) => {
        const next = new Set(prev)
        next.delete(columnId)
        try {
          sessionStorage.setItem(
            boardHiddenStorageKey,
            JSON.stringify([...next])
          )
        } catch {
          /* ignore */
        }
        return next
      })
    },
    [boardHiddenStorageKey]
  )

  const setHideZeroCountBoardColumnsPersisted = React.useCallback(
    (value: boolean) => {
      setHideZeroCountBoardColumns(value)
      try {
        sessionStorage.setItem(hideZeroBoardStorageKey, value ? "1" : "0")
      } catch {
        /* ignore */
      }
    },
    [hideZeroBoardStorageKey]
  )

  const [view, setViewState] = React.useState<"list" | "board">(initialView)
  const setView = React.useCallback((next: "list" | "board") => {
    setViewState(next)
    saveViewPreference(next)
  }, [])

  const scrollContainerRef = React.useRef<HTMLDivElement>(null)
  const hasRestoredFromCacheRef = React.useRef(false)
  const pendingScrollTopRef = React.useRef<number | null>(null)

  // İlk yükleme: cache'den restore et veya API'den yükle
  React.useEffect(() => {
    // Cache'den restore edildiyse Strict Mode ikinci çalışmasında hiçbir şey yapma
    if (hasRestoredFromCacheRef.current) {
      return
    }

    let cancelled = false

    async function init() {
      // Cache'den restore dene
      const savedState = loadScrollState(filterType)
      if (savedState && savedState.items.length > 0) {
        hasRestoredFromCacheRef.current = true
        setItems(savedState.items)
        setNextCursor(savedState.nextCursor)
        setTotal(savedState.total)
        setHasMore(savedState.hasMore)
        setFacets(savedState.facets)
        setIsInitialLoading(false)
        pendingScrollTopRef.current = savedState.scrollTop
        return
      }

      // Cache yoksa API'den yükle
      try {
        setIsInitialLoading(true)
        setError(null)
        setItems([])
        setNextCursor(null)
        setHasMore(true)

        const params = buildFetchParams(
          orgSlug,
          sorting,
          columnFilters,
          null,
          filterType
        )
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

    init()
    return () => {
      cancelled = true
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filterType, orgSlug])

  // Scroll pozisyonunu restore et
  React.useEffect(() => {
    if (pendingScrollTopRef.current !== null && !isInitialLoading && items.length > 0) {
      const scrollTop = pendingScrollTopRef.current
      pendingScrollTopRef.current = null
      requestAnimationFrame(() => {
        scrollContainerRef.current?.scrollTo({ top: scrollTop })
        setScrollTop(scrollTop)
      })
    }
  }, [isInitialLoading, items.length])

  // State'i sessionStorage'a kaydet
  React.useEffect(() => {
    if (items.length > 0 && !isInitialLoading) {
      saveScrollState(filterType, {
        scrollTop,
        items,
        nextCursor,
        total,
        hasMore,
        facets,
      })
    }
  }, [scrollTop, items, nextCursor, total, hasMore, facets, filterType, isInitialLoading])

  // Sonraki sayfa: cursor ile fetch, listeye ekle
  const loadMore = React.useCallback(async () => {
    if (isFetchingMore || !hasMore || !nextCursor) return

    try {
      setIsFetchingMore(true)

      const params = buildFetchParams(
        orgSlug,
        sorting,
        columnFilters,
        nextCursor,
        filterType
      )
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
  }, [
    orgSlug,
    nextCursor,
    hasMore,
    isFetchingMore,
    sorting,
    columnFilters,
    filterType,
  ])

  // Board görünümünde tüm veriyi yükle (liste pagination ile devam eder)
  React.useEffect(() => {
    if (view !== "board" || !hasMore || isFetchingMore || !nextCursor) return
    void loadMore()
  }, [view, hasMore, isFetchingMore, nextCursor, loadMore])

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

  // Sort/filter değişince cache temizle, scroll başa al ve yeniden yükle
  const prevSortingRef = React.useRef(sorting)
  const prevFiltersRef = React.useRef(columnFilters)

  React.useEffect(() => {
    const sortingChanged = JSON.stringify(prevSortingRef.current) !== JSON.stringify(sorting)
    const filtersChanged = JSON.stringify(prevFiltersRef.current) !== JSON.stringify(columnFilters)

    if (sortingChanged || filtersChanged) {
      prevSortingRef.current = sorting
      prevFiltersRef.current = columnFilters

      clearScrollState(filterType)
      scrollContainerRef.current?.scrollTo({ top: 0 })
      setScrollTop(0)

      // Yeniden yükle
      let cancelled = false

      setItems([])
      setNextCursor(null)
      setHasMore(true)

      async function reload() {
        try {
          setIsInitialLoading(true)
          setError(null)

          const params = buildFetchParams(
            orgSlug,
            sorting,
            columnFilters,
            null,
            filterType
          )
          const res = await fetchMyIssues(params)

          if (cancelled) return

          setItems(res.items)
          setTotal(res.total)
          setNextCursor(res.next_cursor ?? null)
          setHasMore(res.has_more)
          setFacets(res.facets)
        } catch (e) {
          if (!cancelled) {
            setError(e instanceof Error ? e.message : "Failed to load issues.")
          }
        } finally {
          if (!cancelled) setIsInitialLoading(false)
        }
      }

      reload()
      return () => {
        cancelled = true
      }
    }
  }, [orgSlug, sorting, columnFilters, filterType])

  const rows = table.getRowModel().rows
  const selectedCount = table.getSelectedRowModel().rows.length
  const visibleColumnCount =
    table.getVisibleLeafColumns().length || columns.length
  const { allBoardColumns, boardItems, statusMetaByProjectAndSlug } =
    React.useMemo(() => {
      type StatusMeta = {
        workflow_status_id: string
        status_name: string
        status_category: string
        status_slug: string
        project_id: string
      }
      type FacetRow = {
        workflow_status_id: string
        project_id: string
        name: string
        slug: string
        category: string
        position: number
        count: number
      }
      const facetStatuses = facets?.status ?? []
      const seenFacet = new Set(facetStatuses.map((f) => f.workflow_status_id))
      const orphanFromRows: FacetRow[] = []
      for (const row of rows) {
        const it = row.original
        if (!seenFacet.has(it.workflow_status_id)) {
          orphanFromRows.push({
            workflow_status_id: it.workflow_status_id,
            project_id: it.project_id,
            name: it.status_name,
            slug: it.status,
            category: it.status_category,
            position: 999_999,
            count: 0,
          })
          seenFacet.add(it.workflow_status_id)
        }
      }
      const combined: FacetRow[] = [...facetStatuses, ...orphanFromRows]
      const sorted = [...combined].sort((a, b) => {
        const rc = categoryRank(a.category) - categoryRank(b.category)
        if (rc !== 0) return rc
        if (a.position !== b.position) return a.position - b.position
        return a.name.localeCompare(b.name)
      })

      const bySlug = new Map<string, FacetRow[]>()
      for (const m of sorted) {
        const arr = bySlug.get(m.slug) ?? []
        arr.push(m)
        bySlug.set(m.slug, arr)
      }

      const slugKeys = [...bySlug.keys()].sort((a, b) => {
        const ga = bySlug.get(a)!
        const gb = bySlug.get(b)!
        const ra =
          categoryRank(ga[0].category) - categoryRank(gb[0].category)
        if (ra !== 0) return ra
        const pa = Math.min(...ga.map((x) => x.position))
        const pb = Math.min(...gb.map((x) => x.position))
        if (pa !== pb) return pa - pb
        return a.localeCompare(b)
      })

      const cols = slugKeys.map((slug) => {
        const group = bySlug.get(slug)!
        const representative = group.reduce((best, cur) =>
          cur.count > best.count ? cur : best
        )
        return {
          id: slug,
          label: representative.name,
          icon: iconForIssueCategory(representative.category),
        }
      })

      const byProjectAndSlug = new Map<string, StatusMeta>()
      for (const m of sorted) {
        const pk = `${m.project_id}:${m.slug}`
        if (!byProjectAndSlug.has(pk)) {
          byProjectAndSlug.set(pk, {
            workflow_status_id: m.workflow_status_id,
            status_name: m.name,
            status_category: m.category,
            status_slug: m.slug,
            project_id: m.project_id,
          })
        }
      }

      const grouped: Record<string, Issue[]> = {}
      for (const c of cols) {
        grouped[c.id] = []
      }

      for (const row of rows) {
        const it = row.original
        const slug = it.status
        if (!grouped[slug]) {
          grouped[slug] = []
          cols.push({
            id: slug,
            label: it.status_name,
            icon: iconForIssueCategory(it.status_category),
          })
          const pk = `${it.project_id}:${slug}`
          if (!byProjectAndSlug.has(pk)) {
            byProjectAndSlug.set(pk, {
              workflow_status_id: it.workflow_status_id,
              status_name: it.status_name,
              status_category: it.status_category,
              status_slug: slug,
              project_id: it.project_id,
            })
          }
        }
        grouped[slug].push(it)
      }

      const appended = cols.slice(slugKeys.length)
      if (appended.length > 0) {
        appended.sort((a, b) => {
          const ca =
            rows.find((r) => r.original.status === a.id)?.original
              .status_category ?? ""
          const cb =
            rows.find((r) => r.original.status === b.id)?.original
              .status_category ?? ""
          const rc = categoryRank(ca) - categoryRank(cb)
          if (rc !== 0) return rc
          return a.id.localeCompare(b.id)
        })
        const head = cols.slice(0, slugKeys.length)
        cols.length = 0
        cols.push(...head, ...appended)
      }

      return {
        allBoardColumns: cols,
        boardItems: grouped,
        statusMetaByProjectAndSlug: byProjectAndSlug,
      }
    }, [facets, rows])

  const visibleBoardColumns = React.useMemo(
    () =>
      allBoardColumns.filter((c) => {
        if (hiddenBoardColumnIds.has(c.id)) return false
        if (
          hideZeroCountBoardColumns &&
          (boardItems[c.id]?.length ?? 0) === 0
        ) {
          return false
        }
        return true
      }),
    [
      allBoardColumns,
      boardItems,
      hiddenBoardColumnIds,
      hideZeroCountBoardColumns,
    ]
  )

  const visibleBoardItems = React.useMemo(() => {
    const out: Record<string, Issue[]> = {}
    for (const c of visibleBoardColumns) {
      out[c.id] = boardItems[c.id] ?? []
    }
    return out
  }, [visibleBoardColumns, boardItems])

  const boardHiddenColumnsForToolbar = React.useMemo(() => {
    const labels = new Map(allBoardColumns.map((c) => [c.id, c.label]))
    return [...hiddenBoardColumnIds]
      .map((id) => ({
        id,
        label: labels.get(id) ?? id,
        issueCount: boardItems[id]?.length ?? 0,
      }))
      .sort((a, b) => a.label.localeCompare(b.label))
  }, [allBoardColumns, boardItems, hiddenBoardColumnIds])

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

  const updateListScrollMetrics = React.useCallback(() => {
    const container = scrollContainerRef.current
    if (!container) return
    setContainerHeight(container.clientHeight)
    setListBodyOverflows(container.scrollHeight > container.clientHeight + 1)
  }, [])

  React.useLayoutEffect(() => {
    if (view !== "list") return
    const container = scrollContainerRef.current
    if (!container) return

    updateListScrollMetrics()
    const observer = new ResizeObserver(updateListScrollMetrics)
    observer.observe(container)
    return () => observer.disconnect()
  }, [
    view,
    updateListScrollMetrics,
    rows.length,
    isInitialLoading,
    error,
    topSpacerHeight,
    bottomSpacerHeight,
  ])

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

  const HEADER_HEIGHT = 41
  const listContentHeight = HEADER_HEIGHT + rows.length * ROW_HEIGHT

  const hasActiveIssueFilters = columnFilters.some((f) => {
    const v = f.value
    if (v === undefined || v === "") return false
    if (Array.isArray(v) && v.length === 0) return false
    return true
  })

  const showListEmptyState =
    view === "list" &&
    !isInitialLoading &&
    !error &&
    rows.length === 0
  const listEmptyCopy = myIssuesEmptyCopy(filterType, hasActiveIssueFilters)

  const refetchIssues = React.useCallback(
    async () => {
      try {
        const params = buildFetchParams(
          orgSlug,
          sorting,
          columnFilters,
          null,
          filterType
        )
        const res = await fetchMyIssues(params)
        setItems(res.items)
        setTotal(res.total)
        setNextCursor(res.next_cursor ?? null)
        setHasMore(res.has_more)
        setFacets(res.facets)
      } catch {
        // keep current state on refetch error
      }
    },
    [orgSlug, sorting, columnFilters, filterType]
  )

  const handleBoardMove = React.useCallback(
    async ({
      issue,
      issueId,
      toColumnId,
      columns: columnState,
    }: {
      issue: Issue
      issueId: string
      toColumnId: string
      columns: Record<string, Issue[]>
    }) => {
      const targetSlug = toColumnId
      const metaKey = `${issue.project_id}:${targetSlug}`
      const targetMeta = statusMetaByProjectAndSlug.get(metaKey)
      if (!targetMeta) {
        return false
      }

      const targetColumnOrder = columnState[toColumnId] ?? []
      const sameProjectOrdered = targetColumnOrder.filter(
        (i) => i.project_id === issue.project_id
      )
      const boardPosition = sameProjectOrdered.findIndex((i) => i.id === issueId)
      if (boardPosition < 0) {
        return false
      }

      const previousSnapshot = {
        workflow_status_id: issue.workflow_status_id,
        status_name: issue.status_name,
        status_category: issue.status_category,
        status: issue.status,
      }

      const statusChanged =
        issue.workflow_status_id !== targetMeta.workflow_status_id

      if (statusChanged) {
        setItems((prev) =>
          prev.map((item) =>
            item.id === issueId
              ? {
                  ...item,
                  workflow_status_id: targetMeta.workflow_status_id,
                  status_name: targetMeta.status_name,
                  status_category: targetMeta.status_category,
                  status: targetMeta.status_slug,
                }
              : item
          )
        )
      }

      try {
        const success = await updateIssueBoardPosition(
          issueId,
          targetMeta.workflow_status_id,
          boardPosition
        )

        if (!success) {
          setItems((prev) =>
            prev.map((item) =>
              item.id === issueId ? { ...item, ...previousSnapshot } : item
            )
          )
          return false
        }

        updateIssueInCaches(issue.display_key, {
          workflow_status_id: targetMeta.workflow_status_id,
          status_name: targetMeta.status_name,
          status_category: targetMeta.status_category,
          status: targetMeta.status_slug,
        })
        await refetchIssues()
        return true
      } catch {
        setItems((prev) =>
          prev.map((item) =>
            item.id === issueId ? { ...item, ...previousSnapshot } : item
          )
        )
        return false
      }
    },
    [refetchIssues, statusMetaByProjectAndSlug]
  )

  const canIssueEnterBoardColumn = React.useCallback(
    (issue: Issue, columnId: string) =>
      statusMetaByProjectAndSlug.has(`${issue.project_id}:${columnId}`),
    [statusMetaByProjectAndSlug]
  )

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <DataTableToolbar
        table={table}
        facets={facets}
        view={view}
        onViewChange={setView}
        boardHiddenColumns={boardHiddenColumnsForToolbar}
        onUnhideBoardColumn={unhideBoardColumn}
        hideZeroCountBoardColumns={hideZeroCountBoardColumns}
        onHideZeroCountBoardColumnsChange={
          setHideZeroCountBoardColumnsPersisted
        }
      />

      <div className="relative flex max-h-full min-h-0 flex-1 flex-col overflow-hidden">
        {view === "list" ? (
          <>
            <DataTableSelectionOverlay
              selectedCount={selectedCount}
              onClearSelection={() => table.resetRowSelection()}
            />

            <div
              className={`relative flex flex-col overflow-hidden rounded-md border${showListEmptyState ? " min-h-[140px]" : ""}`}
              style={{ height: `min(${listContentHeight}px, 100%)` }}
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
                className={
                  listBodyOverflows
                    ? "relative min-h-0 flex-1 overflow-y-auto"
                    : "relative min-h-0 flex-1 overflow-y-hidden"
                }
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
                      <TableRow className="hover:bg-transparent">
                        <TableCell
                          colSpan={visibleColumnCount}
                          className="h-auto min-h-[120px] align-middle px-6 py-6"
                        >
                          <div className="flex flex-col items-center justify-center gap-1.5 text-center">
                            <p className="text-sm font-medium text-foreground">
                              {listEmptyCopy.title}
                            </p>
                            <p className="max-w-sm text-xs text-muted-foreground">
                              {listEmptyCopy.description}
                            </p>
                          </div>
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </table>
              </div>
            </div>
          </>
        ) : !isInitialLoading &&
          !error &&
          visibleBoardColumns.length === 0 &&
          allBoardColumns.length > 0 ? (
          <div className="flex min-h-[140px] flex-1 flex-col items-center justify-center gap-1.5 rounded-md border border-dashed px-5 py-6 text-center">
            <p className="text-sm font-medium text-foreground">
              No board columns visible
            </p>
            <p className="max-w-md text-xs text-muted-foreground">
              Use <strong className="text-foreground">Columns</strong> next to
              View to show hidden or zero-count columns again.
            </p>
          </div>
        ) : !isInitialLoading && !error && allBoardColumns.length === 0 ? (
          <div className="flex min-h-[140px] flex-1 flex-col items-center justify-center gap-1.5 rounded-md border border-dashed px-5 py-6 text-center">
            <p className="text-sm font-medium text-foreground">
              {listEmptyCopy.title}
            </p>
            <p className="max-w-md text-xs text-muted-foreground">
              {listEmptyCopy.description}
            </p>
          </div>
        ) : (
          <div className="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
            <IssueKanbanBoard
              columns={visibleBoardColumns}
              itemsByColumn={visibleBoardItems}
              isLoading={isInitialLoading}
              error={error}
              emptyMessage={
                hasActiveIssueFilters
                  ? "No issues match filters"
                  : filterType === "created"
                    ? "No issues created by you"
                    : "Nothing assigned to you"
              }
              getIssueId={(issue) => issue.id}
              getIssueKey={(issue) => issue.display_key}
              getIssueTitle={(issue) => issue.title}
              getIssueHref={(issue) =>
                issueDetailHref(orgSlug, issue.display_key, { from: "my-issues" })
              }
              getIssueProjectId={(issue) => issue.project_id}
              canIssueEnterColumn={canIssueEnterBoardColumn}
              onHideColumn={hideBoardColumn}
              onIssueMove={({
                issue,
                issueId,
                toColumnId,
                columns: moveColumns,
              }) =>
                handleBoardMove({
                  issue,
                  issueId,
                  toColumnId,
                  columns: moveColumns,
                })
              }
              onReload={refetchIssues}
              onAddIssue={(columnId) => newIssueModal?.openNewIssueModal(columnId)}
            />
          </div>
        )}
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

