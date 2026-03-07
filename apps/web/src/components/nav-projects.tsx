"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { MoreHorizontalIcon, PlusIcon } from "lucide-react"

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
      <div className="flex items-center">
        <SidebarGroupLabel className="flex-1">Projects</SidebarGroupLabel>
        {projects.length > 0 && (<SidebarGroupAction
          aria-label="Create project"
          title="Create project"
          type="button"
          className="rounded-sm cursor-pointer relative top-0 right-0"
        >
          <PlusIcon />
        </SidebarGroupAction>)}
      </div>
      {projects.length === 0 ? (<NavProjectsEmpty />) : (<NavProjectsList projects={projects} />)}
    </SidebarGroup>
  )
}
function NavProjectsEmpty() {
  return (
    <SidebarMenu className="flex flex-col items-center justify-center">
    <SidebarMenuItem>
      <p className="px-2 pb-3 text-xs text-sidebar-foreground/70">
        No projects yet.
      </p>
    </SidebarMenuItem>
    <NavProjectsCreate />
    </SidebarMenu>
  )
}

function NavProjectsCreate() {
  return (
    <SidebarMenuItem className="border border-dashed border-sidebar-border hover:bg-sidebar-accent hover:border-sidebar-accent w-full rounded-sm">
      <SidebarMenuButton
        className="flex items-center justify-center cursor-pointer"
        type="button"
        onClick={() => {
          // TODO: Open create project modal in the future
        }}
      >
        <PlusIcon />
        <span>Create project</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}

function NavProjectsList({ projects }: { projects: {
  name: string
  key: string
  url: string
  icon: React.ReactNode
  openIssueCount: number
}[]}) {
  return (
    <SidebarMenu>
    {projects.map((project) => (
      <NavProjectsItem key={project.name} project={project} />
    ))}
    <NavProjectsMore />
    </SidebarMenu>
  )
}

function NavProjectsItem({ project }: { project: {
  name: string
  key: string
  url: string
  icon: React.ReactNode
  openIssueCount: number
}}) {
  const pathname = usePathname()

  function isRouteActive(url: string) {
    const route = url.split("#")[0]
    return pathname === route || pathname.startsWith(`${route}/`)
  }

  return (
    <SidebarMenuItem>
      <SidebarMenuButton asChild isActive={isRouteActive(project.url)} className="h-auto py-2">
        <Link href={project.url}>
          {project.icon}
          <div className="flex min-w-0 flex-1 flex-col text-left">
            <span className="truncate font-medium">{project.name}</span>
            <span className="truncate text-[11px] text-sidebar-foreground/60">
              {project.key}
            </span>
          </div>
          <SidebarMenuBadge>{project.openIssueCount}</SidebarMenuBadge>
        </Link>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}

function NavProjectsMore() {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton className="text-sidebar-foreground/60">
        <MoreHorizontalIcon className="text-sidebar-foreground/60"/>
        <span>More</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}