"use client"

import * as React from "react"

import { NavMain } from "@/components/nav-main"
import { NavProjects } from "@/components/nav-projects"
import { NavUser } from "@/components/nav-user"
import { OrganizationSwitcher } from "@/components/organization-switcher"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"
import { GalleryVerticalEndIcon, AudioLinesIcon, TerminalIcon, TerminalSquareIcon, BotIcon, BookOpenIcon, Settings2Icon, FrameIcon, PieChartIcon, MapIcon, Layers, Crosshair, Boxes, Settings, BlocksIcon, KeyRoundIcon, WorkflowIcon } from "lucide-react"

// This is sample data.
const data = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
  organizations: [
    {
      name: "Acme Inc",
      logo: (
        <GalleryVerticalEndIcon
        />
      ),
      plan: "Enterprise",
    },
    {
      name: "Acme Corp.",
      logo: (
        <AudioLinesIcon
        />
      ),
      plan: "Startup",
    },
    {
      name: "Evil Corp.",
      logo: (
        <TerminalIcon
        />
      ),
      plan: "Free",
    },
  ],
  navMain: [
    {
      title: "My Issues",
      url: "#",
      icon: (
        <Crosshair
        />
      )
    },
    {
      title: "Projects",
      url: "#",
      icon: (
        <Boxes
        />
      ),
    },
    {
      title: "Activity",
      url: "#",
      icon: (
        <BookOpenIcon
        />
      ),
    },
    {
      title: "Settings",
      url: "#",
      icon: (
        <Settings
        />
      ),
    },
  ],
  projects: [
    {
      name: "Core Platform",
      key: "KORO",
      url: "/dashboard/projects#koro",
      icon: <BlocksIcon />,
      openIssueCount: 24,
    },
    {
      name: "Authentication",
      key: "AUTH",
      url: "/dashboard/projects#auth",
      icon: <KeyRoundIcon />,
      openIssueCount: 9,
    },
    {
      name: "Workflow Engine",
      key: "FLOW",
      url: "/dashboard/projects#flow",
      icon: <WorkflowIcon />,
      openIssueCount: 13,
    },
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <OrganizationSwitcher organizations={data.organizations} />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
