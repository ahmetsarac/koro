"use client"

import * as React from "react"
import { use } from "react"
import Link from "next/link"
import {
  FolderKanban,
  Users,
  FileText,
  ChevronRight,
  Plus,
  LayoutGrid,
  List,
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { projectRoles } from "@/app/org/[orgSlug]/projects/data/data"
import { Skeleton } from "@/components/ui/skeleton"

interface Project {
  id: string
  project_key: string
  name: string
  description: string | null
  org_id: string
  org_name: string
  org_slug: string
  issue_count: number
  member_count: number
  my_role: string
  created_at: string
}

async function fetchProject(
  orgSlug: string,
  projectKey: string
): Promise<Project> {
  const response = await fetch(`/api/orgs/${orgSlug}/projects/${projectKey}`, {
    cache: "no-store",
    credentials: "same-origin",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch project")
  }

  return response.json()
}

export default function ProjectDetailPage({
  params,
}: {
  params: Promise<{ orgSlug: string; projectKey: string }>
}) {
  const { orgSlug, projectKey } = use(params)
  const [project, setProject] = React.useState<Project | null>(null)
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    async function load() {
      try {
        setIsLoading(true)
        const data = await fetchProject(orgSlug, projectKey)
        setProject(data)
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load project")
      } finally {
        setIsLoading(false)
      }
    }

    load()
  }, [orgSlug, projectKey])

  if (isLoading) {
    return (
      <div className="flex flex-col gap-4">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Skeleton className="h-4 w-16" />
          <ChevronRight className="h-4 w-4" />
          <Skeleton className="h-4 w-24" />
        </div>
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-96" />
        <Skeleton className="h-10 w-full mt-4" />
      </div>
    )
  }

  if (error || !project) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="text-center">
          <h2 className="text-lg font-semibold text-destructive">Error</h2>
          <p className="text-muted-foreground">{error || "Project not found"}</p>
          <Button asChild className="mt-4">
            <Link href={`/org/${orgSlug}/projects`}>Back to Projects</Link>
          </Button>
        </div>
      </div>
    )
  }

  const role = projectRoles.find((r) => r.value === project.my_role)

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Link
          href={`/org/${orgSlug}/projects`}
          className="hover:text-foreground transition-colors"
        >
          Projects
        </Link>
        <ChevronRight className="h-4 w-4" />
        <span className="font-mono">{project.project_key}</span>
      </div>

      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-3">
            <FolderKanban className="h-6 w-6 text-muted-foreground" />
            <h1 className="text-2xl font-semibold">{project.name}</h1>
          </div>
          {project.description && (
            <p className="text-muted-foreground mt-1 max-w-2xl">
              {project.description}
            </p>
          )}
        </div>

        <Button data-icon="inline-start">
          <Plus className="h-4 w-4" />
          New Issue
        </Button>
      </div>

      <div className="flex items-center gap-6 text-sm">
        <div className="flex items-center gap-1.5 text-muted-foreground">
          <FileText className="h-4 w-4" />
          <span>
            {project.issue_count} issue{project.issue_count !== 1 && "s"}
          </span>
        </div>
        <div className="flex items-center gap-1.5 text-muted-foreground">
          <Users className="h-4 w-4" />
          <span>
            {project.member_count} member{project.member_count !== 1 && "s"}
          </span>
        </div>
        {role && (
          <div className="flex items-center gap-1.5">
            {role.icon && <role.icon className="h-4 w-4 text-muted-foreground" />}
            <span className="text-muted-foreground">{role.label}</span>
          </div>
        )}
      </div>

      <Tabs defaultValue="issues" className="flex-1 mt-2">
        <TabsList>
          <TabsTrigger value="issues" className="gap-2">
            <List className="h-4 w-4" />
            Issues
          </TabsTrigger>
          <TabsTrigger value="board" className="gap-2">
            <LayoutGrid className="h-4 w-4" />
            Board
          </TabsTrigger>
          <TabsTrigger value="members" className="gap-2">
            <Users className="h-4 w-4" />
            Members
          </TabsTrigger>
        </TabsList>

        <TabsContent value="issues" className="mt-4">
          <ProjectIssuesTab orgSlug={orgSlug} projectKey={projectKey} />
        </TabsContent>

        <TabsContent value="board" className="mt-4">
          <div className="text-center text-muted-foreground py-12 border rounded-md">
            Board view coming soon...
          </div>
        </TabsContent>

        <TabsContent value="members" className="mt-4">
          <div className="text-center text-muted-foreground py-12 border rounded-md">
            Members view coming soon...
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}

function ProjectIssuesTab({
  orgSlug,
  projectKey,
}: {
  orgSlug: string
  projectKey: string
}) {
  return (
    <div className="text-center text-muted-foreground py-12 border rounded-md">
      <p>Issues for {projectKey} will be listed here.</p>
      <p className="text-sm mt-2">
        This will use the existing issues endpoint at{" "}
        <code className="bg-muted px-1 py-0.5 rounded">
          /projects/{"{projectId}"}/issues
        </code>
      </p>
    </div>
  )
}
