"use client"

import * as React from "react"
import Link from "next/link"
import { usePathname, useSearchParams } from "next/navigation"
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"
import { projectKeyFromIssueKey } from "@/lib/issue-nav"

const routeLabels: Record<string, string> = {
  issue: "Issue",
  "my-issues": "My Issues",
  assigned: "Assigned",
  created: "Created",
  projects: "Projects",
  settings: "Settings",
  "workflow-statuses": "Workflow statuses",
}

type Crumb = { label: string; href: string; isLast: boolean }

function OrgBreadcrumbInner() {
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const segments = pathname.split("/").filter(Boolean)

  if (segments.length < 2) {
    return null
  }

  const orgSlug = segments[0]
  const pathSegments = segments.slice(1)

  let breadcrumbs: Crumb[] = []

  if (pathSegments.length === 2 && pathSegments[0] === "issue") {
    const issueKey = pathSegments[1]
    const from = searchParams.get("from")
    const projectParam = searchParams.get("project")
    const projectKeyResolved =
      projectParam?.trim() || projectKeyFromIssueKey(issueKey)

    if (from === "my-issues") {
      breadcrumbs = [
        {
          label: routeLabels["my-issues"],
          href: `/${orgSlug}/my-issues`,
          isLast: false,
        },
        { label: issueKey, href: `${pathname}?${searchParams}`, isLast: true },
      ]
    } else if (from === "project") {
      breadcrumbs = [
        {
          label: routeLabels.projects,
          href: `/${orgSlug}/projects`,
          isLast: false,
        },
        {
          label: projectKeyResolved,
          href: `/${orgSlug}/projects/${encodeURIComponent(projectKeyResolved)}`,
          isLast: false,
        },
        { label: issueKey, href: `${pathname}?${searchParams}`, isLast: true },
      ]
    }
  }

  if (breadcrumbs.length === 0) {
    let currentPath = `/${orgSlug}`
    pathSegments.forEach((segment, index) => {
      currentPath += `/${segment}`
      const isLast = index === pathSegments.length - 1
      const label = routeLabels[segment] || segment.toUpperCase()

      breadcrumbs.push({
        label,
        href: currentPath,
        isLast,
      })
    })
  }

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {breadcrumbs.map((crumb, index) => (
          <React.Fragment key={`${crumb.href}-${index}`}>
            {index > 0 && <BreadcrumbSeparator className="mx-1" />}
            <BreadcrumbItem>
              {crumb.isLast ? (
                <BreadcrumbPage>{crumb.label}</BreadcrumbPage>
              ) : (
                <BreadcrumbLink asChild>
                  <Link href={crumb.href}>{crumb.label}</Link>
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
          </React.Fragment>
        ))}
      </BreadcrumbList>
    </Breadcrumb>
  )
}

export function OrgBreadcrumb() {
  return (
    <React.Suspense fallback={null}>
      <OrgBreadcrumbInner />
    </React.Suspense>
  )
}
