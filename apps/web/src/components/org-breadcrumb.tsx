"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb"

const routeLabels: Record<string, string> = {
  "my-issues": "My Issues",
  assigned: "Assigned",
  created: "Created",
  projects: "Projects",
  activity: "Activity",
  settings: "Settings",
}

export function OrgBreadcrumb() {
  const pathname = usePathname()
  const segments = pathname.split("/").filter(Boolean)

  const orgIndex = segments.indexOf("org")
  if (orgIndex === -1 || orgIndex + 1 >= segments.length) {
    return null
  }

  const orgSlug = segments[orgIndex + 1]
  const pathSegments = segments.slice(orgIndex + 2)

  if (pathSegments.length === 0) {
    return null
  }

  const breadcrumbs: { label: string; href: string; isLast: boolean }[] = []
  let currentPath = `/org/${orgSlug}`

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

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {breadcrumbs.map((crumb, index) => (
          <BreadcrumbItem key={crumb.href}>
            {index > 0 && <BreadcrumbSeparator className="mx-1" />}
            {crumb.isLast ? (
              <BreadcrumbPage>{crumb.label}</BreadcrumbPage>
            ) : (
              <BreadcrumbLink asChild>
                <Link href={crumb.href}>{crumb.label}</Link>
              </BreadcrumbLink>
            )}
          </BreadcrumbItem>
        ))}
      </BreadcrumbList>
    </Breadcrumb>
  )
}
