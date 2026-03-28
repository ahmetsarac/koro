import { cookies } from "next/headers"

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
import { Crosshair, Boxes, Building2 } from "lucide-react"
import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"
import { meResponseSchema } from "@/lib/user"
import { projectListResponseSchema } from "@/app/[orgSlug]/projects/data/schema"

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
  ]
}

async function fetchSidebarData(orgSlug: string) {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get(ACCESS_TOKEN_COOKIE_NAME)?.value

  if (!accessToken) {
    return {
      me: null,
      projects: [],
    }
  }

  const headers = {
    Authorization: `Bearer ${accessToken}`,
  }

  const [meResponse, projectsResponse] = await Promise.all([
    fetch(`${getApiBaseUrl()}/me`, {
      cache: "no-store",
      headers,
    }),
    fetch(`${getApiBaseUrl()}/projects?limit=8`, {
      cache: "no-store",
      headers,
    }),
  ])

  const me = meResponse.ok
    ? meResponseSchema.parse(await meResponse.json())
    : null

  const projects = projectsResponse.ok
    ? projectListResponseSchema
        .parse(await projectsResponse.json())
        .items.filter((project) => project.org_slug === orgSlug)
    : []

  return { me, projects }
}

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  orgSlug: string
}

export async function AppSidebar({ orgSlug, ...props }: AppSidebarProps) {
  const { me, projects: sidebarProjects } = await fetchSidebarData(orgSlug)
  const navItems = getNavItems(orgSlug)

  const organizations =
    me?.organizations.map((org) => ({
      id: org.id,
      name: org.name,
      slug: org.slug,
      role: org.role,
      logo: <Building2 className="size-4" />,
    })) ?? []

  const projects = sidebarProjects.map((project) => ({
    name: project.name,
    key: project.project_key,
    url: `/${project.org_slug}/projects/${project.project_key}`,
    icon: <Boxes className="size-4" />,
    openIssueCount: project.issue_count,
  }))

  const user = me
    ? {
        name: me.name,
        email: me.email,
        avatar: "",
      }
    : {
        name: "Unknown user",
        email: "",
        avatar: "",
      }

  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <OrganizationSwitcher
          organizations={organizations}
          currentOrgSlug={orgSlug}
        />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={navItems} />
        <NavProjects projects={projects} orgSlug={orgSlug} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
