// Shared in-memory cache for issues lists
// Cleared on page refresh, persists during client-side navigation

import type { ColumnFiltersState, SortingState } from "@tanstack/react-table"

export interface MyIssuesScrollState {
  scrollTop: number
  items: unknown[]
  nextCursor: string | null
  total: number
  hasMore: boolean
  facets: unknown
  sorting?: SortingState
  columnFilters?: ColumnFiltersState
}

export interface ProjectIssuesScrollState {
  scrollTop: number
  items: unknown[]
  total: number
  hasMore: boolean
}

export type IssueFilterType = "assigned" | "created"

// My Issues cache (assigned/created tabs)
export const myIssuesCache = new Map<IssueFilterType, MyIssuesScrollState>()

// Project Issues cache (per org-project)
export const projectIssuesCache = new Map<string, ProjectIssuesScrollState>()

// Clear all caches (call after issue update)
export function clearAllIssuesCaches(): void {
  myIssuesCache.clear()
  projectIssuesCache.clear()
}

/** My Issues view preference key in localStorage (list vs board). */
export const MY_ISSUES_VIEW_STORAGE_KEY = "koro_my_issues_view"

/**
 * Call on logout (or user change). Clears in-memory caches and my-issues view
 * preference so the next user does not see the previous user's data.
 */
export function clearUserIssuesDataOnLogout(): void {
  myIssuesCache.clear()
  projectIssuesCache.clear()
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(MY_ISSUES_VIEW_STORAGE_KEY)
  }
}

// Clear specific project issues cache
export function clearProjectIssuesCache(orgSlug: string, projectKey: string): void {
  projectIssuesCache.delete(`${orgSlug}-${projectKey}`)
}

// Update a specific issue in all caches
export function updateIssueInCaches(
  issueKey: string,
  updates: {
    title?: string
    description?: string
    status?: string
    workflow_status_id?: string
    status_name?: string
    status_category?: string
    is_blocked?: boolean
    priority?: string
    assignee_id?: string | null
    assignee_name?: string | null
  }
): void {
  // Update in myIssuesCache
  for (const [filterType, state] of myIssuesCache.entries()) {
    const items = state.items as Array<{
      display_key?: string
      title?: string
      description?: string
      status?: string
      workflow_status_id?: string
      status_name?: string
      status_category?: string
      is_blocked?: boolean
      priority?: string
      assignee_id?: string | null
      assignee_name?: string | null
    }>
    const index = items.findIndex((item) => item.display_key === issueKey)
    if (index !== -1) {
      const updatedItems = [...items]
      updatedItems[index] = { ...updatedItems[index], ...updates }
      myIssuesCache.set(filterType, { ...state, items: updatedItems })
    }
  }

  // Update in projectIssuesCache
  for (const [cacheKey, state] of projectIssuesCache.entries()) {
    const items = state.items as Array<{
      display_key?: string
      title?: string
      description?: string
      status?: string
      workflow_status_id?: string
      status_name?: string
      status_category?: string
      is_blocked?: boolean
      priority?: string
      assignee_id?: string | null
      assignee_name?: string | null
    }>
    const index = items.findIndex((item) => item.display_key === issueKey)
    if (index !== -1) {
      const updatedItems = [...items]
      updatedItems[index] = { ...updatedItems[index], ...updates }
      projectIssuesCache.set(cacheKey, { ...state, items: updatedItems })
    }
  }
}
