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
import {
  GalleryVerticalEndIcon,
  AudioLinesIcon,
  TerminalIcon,
  BookOpenIcon,
  Crosshair,
  Boxes,
  Settings,
  BlocksIcon,
  KeyRoundIcon,
  WorkflowIcon,
} from "lucide-react"

const staticData = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
  organizations: [
    {
      name: "Acme Inc",
      slug: "acme",
      logo: <GalleryVerticalEndIcon />,
      plan: "Enterprise",
    },
    {
      name: "Acme Corp.",
      slug: "acme-corp",
      logo: <AudioLinesIcon />,
      plan: "Startup",
    },
    {
      name: "Evil Corp.",
      slug: "evil-corp",
      logo: <TerminalIcon />,
      plan: "Free",
    },
  ],
}

function getNavItems(orgSlug: string) {
  return [
    {
      title: "My Issues",
      url: `/${orgSlug}/my-issues`,
      icon: <Crosshair />,
    },
    {
      title: "Projects",
      url: `/${orgSlug}/projects`,
      icon: <Boxes />,
    },
    {
      title: "Activity",
      url: `/${orgSlug}/activity`,
      icon: <BookOpenIcon />,
    },
    {
      title: "Settings",
      url: `/${orgSlug}/settings`,
      icon: <Settings />,
    },
  ]
}

function getProjects(orgSlug: string) {
  return [
    {
      name: "Core Platform",
      key: "KORO",
      url: `/${orgSlug}/projects/KORO`,
      icon: <BlocksIcon />,
      openIssueCount: 24,
    },
    {
      name: "Authentication",
      key: "AUTH",
      url: `/${orgSlug}/projects/AUTH`,
      icon: <KeyRoundIcon />,
      openIssueCount: 9,
    },
    {
      name: "Workflow Engine",
      key: "FLOW",
      url: `/${orgSlug}/projects/FLOW`,
      icon: <WorkflowIcon />,
      openIssueCount: 13,
    },
  ]
}

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  orgSlug: string
}

export function AppSidebar({ orgSlug, ...props }: AppSidebarProps) {
  const navItems = getNavItems(orgSlug)
  const projects = getProjects(orgSlug)

  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <OrganizationSwitcher
          organizations={staticData.organizations}
          currentOrgSlug={orgSlug}
        />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={navItems} />
        <NavProjects projects={projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={staticData.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
