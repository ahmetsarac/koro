"use client"

import * as React from "react"
import Link from "next/link"
import { usePathname, useRouter } from "next/navigation"
import { ChevronDownIcon, ChevronUpIcon, PlusIcon } from "lucide-react"

import { NewProjectModal } from "@/components/projects/new-project-modal"
import { cn } from "@/lib/utils"
import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

const NAV_PROJECTS_MENU_ID = "nav-projects-menu"
const INITIAL_VISIBLE_PROJECTS = 3

export function NavProjects({
  projects,
  orgSlug,
}: {
  orgSlug: string
  projects: {
    name: string
    key: string
    url: string
    icon: React.ReactNode
    openIssueCount: number
  }[]
}) {
  const router = useRouter()
  const [createProjectOpen, setCreateProjectOpen] = React.useState(false)

  return (
    <>
      <NewProjectModal
        orgSlug={orgSlug}
        open={createProjectOpen}
        onOpenChange={setCreateProjectOpen}
        onSuccess={() => router.refresh()}
      />
      <SidebarGroup
        className={cn(
          "group-data-[collapsible=icon]:hidden",
          projects.length > 0 && "flex min-h-0 flex-1 flex-col"
        )}
      >
        <div
          className={cn(
            "flex items-center",
            projects.length > 0 && "shrink-0"
          )}
        >
          <SidebarGroupLabel className="flex-1">Projects</SidebarGroupLabel>
          {projects.length > 0 && (
            <SidebarGroupAction
              aria-label="Create project"
              title="Create project"
              type="button"
              className="rounded-sm cursor-pointer relative top-0 right-0"
              onClick={() => setCreateProjectOpen(true)}
            >
              <PlusIcon />
            </SidebarGroupAction>
          )}
        </div>
        {projects.length === 0 ? (
          <NavProjectsEmpty onCreateProject={() => setCreateProjectOpen(true)} />
        ) : (
          <NavProjectsList projects={projects} />
        )}
      </SidebarGroup>
    </>
  )
}
function NavProjectsEmpty({
  onCreateProject,
}: {
  onCreateProject: () => void
}) {
  return (
    <SidebarMenu className="flex flex-col items-center justify-center">
    <SidebarMenuItem>
      <p className="px-2 pb-3 text-xs text-sidebar-foreground/70">
        No projects yet.
      </p>
    </SidebarMenuItem>
    <NavProjectsCreate onCreateProject={onCreateProject} />
    </SidebarMenu>
  )
}

function NavProjectsCreate({
  onCreateProject,
}: {
  onCreateProject: () => void
}) {
  return (
    <SidebarMenuItem className="border border-dashed border-sidebar-border hover:bg-sidebar-accent hover:border-sidebar-accent w-full rounded-sm">
      <SidebarMenuButton
        className="flex items-center justify-center cursor-pointer"
        type="button"
        onClick={onCreateProject}
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
  const [expanded, setExpanded] = React.useState(false)
  const showToggle = projects.length > INITIAL_VISIBLE_PROJECTS
  const visibleProjects =
    showToggle && !expanded
      ? projects.slice(0, INITIAL_VISIBLE_PROJECTS)
      : projects

  return (
    <div
      id={NAV_PROJECTS_MENU_ID}
      className="min-h-0 flex-1 overflow-y-auto overscroll-contain"
    >
      <SidebarMenu>
        {visibleProjects.map((project) => (
          <NavProjectsItem key={project.key} project={project} />
        ))}
        {showToggle ? (
          <NavProjectsToggle
            expanded={expanded}
            onToggle={() => setExpanded((v) => !v)}
          />
        ) : null}
      </SidebarMenu>
    </div>
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

function NavProjectsToggle({
  expanded,
  onToggle,
}: {
  expanded: boolean
  onToggle: () => void
}) {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        type="button"
        className="text-sidebar-foreground/60"
        onClick={onToggle}
        aria-expanded={expanded}
        aria-controls={NAV_PROJECTS_MENU_ID}
      >
        {expanded ? (
          <ChevronUpIcon className="text-sidebar-foreground/60" />
        ) : (
          <ChevronDownIcon className="text-sidebar-foreground/60" />
        )}
        <span>{expanded ? "Less" : "More"}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}