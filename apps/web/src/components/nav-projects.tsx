"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { PlusIcon } from "lucide-react"

import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

export function NavProjects({
  projects,
}: {
  projects: {
    name: string
    key: string
    url: string
    icon: React.ReactNode
    openIssueCount: number
  }[]
}) {
  const pathname = usePathname()

  function isRouteActive(url: string) {
    const route = url.split("#")[0]
    return pathname === route || pathname.startsWith(`${route}/`)
  }

  return (
    <SidebarGroup className="group-data-[collapsible=icon]:hidden">
      <SidebarGroupAction
        aria-label="Create project"
        title="Create project"
        type="button"
        className="rounded-sm cursor-pointer"
      >
        <PlusIcon />
      </SidebarGroupAction>
      <SidebarGroupLabel>Projects</SidebarGroupLabel>
      <SidebarMenu>
        {projects.map((item) => (
          <SidebarMenuItem key={item.name}>
            <SidebarMenuButton asChild isActive={isRouteActive(item.url)} className="h-auto py-2">
              <Link href={item.url}>
                {item.icon}
                <div className="flex min-w-0 flex-1 flex-col text-left">
                  <span className="truncate font-medium">{item.name}</span>
                  <span className="truncate text-[11px] text-sidebar-foreground/60">
                    {item.key}
                  </span>
                </div>
                <SidebarMenuBadge>{item.openIssueCount}</SidebarMenuBadge>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  )
}