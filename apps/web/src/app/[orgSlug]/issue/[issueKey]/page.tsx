"use client"

import * as React from "react"
import { use } from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"
import {
  Circle,
  Timer,
  CheckCircle,
  HelpCircle,
  Ban,
  User,
  Calendar,
  Clock,
  ArrowLeft,
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"

interface Issue {
  issue_id: string
  display_key: string
  title: string
  description: string | null
  status: string
  assignee_id: string | null
  project_id: string
  created_at: string
  updated_at: string
}

const statusConfig: Record<
  string,
  { label: string; icon: React.ElementType; color: string }
> = {
  backlog: { label: "Backlog", icon: HelpCircle, color: "bg-muted text-muted-foreground" },
  todo: { label: "Todo", icon: Circle, color: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300" },
  in_progress: { label: "In Progress", icon: Timer, color: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300" },
  blocked: { label: "Blocked", icon: Ban, color: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300" },
  done: { label: "Done", icon: CheckCircle, color: "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300" },
}

async function fetchIssue(orgSlug: string, issueKey: string): Promise<Issue> {
  const response = await fetch(`/api/orgs/${orgSlug}/issues/${issueKey}`, {
    cache: "no-store",
    credentials: "same-origin",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch issue")
  }

  return response.json()
}

export default function IssueDetailPage({
  params,
}: {
  params: Promise<{ orgSlug: string; issueKey: string }>
}) {
  const { orgSlug, issueKey } = use(params)
  const router = useRouter()
  const [issue, setIssue] = React.useState<Issue | null>(null)
  const [isLoading, setIsLoading] = React.useState(true)
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    async function load() {
      try {
        setIsLoading(true)
        const data = await fetchIssue(orgSlug, issueKey)
        setIssue(data)
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load issue")
      } finally {
        setIsLoading(false)
      }
    }

    load()
  }, [orgSlug, issueKey])

  if (isLoading) {
    return (
      <div className="flex h-[calc(100svh-4.5rem)] flex-col gap-6">
        <div className="flex items-center gap-4">
          <Skeleton className="h-8 w-8" />
          <Skeleton className="h-6 w-24" />
        </div>
        <Skeleton className="h-10 w-2/3" />
        <Skeleton className="h-6 w-32" />
        <Skeleton className="h-40 w-full" />
      </div>
    )
  }

  if (error || !issue) {
    return (
      <div className="flex h-[calc(100svh-4.5rem)] flex-1 items-center justify-center">
        <div className="text-center">
          <h2 className="text-lg font-semibold text-destructive">Error</h2>
          <p className="text-muted-foreground">{error || "Issue not found"}</p>
          <Button asChild className="mt-4">
            <Link href={`/${orgSlug}/my-issues`}>Back to Issues</Link>
          </Button>
        </div>
      </div>
    )
  }

  const status = statusConfig[issue.status] || {
    label: issue.status,
    icon: Circle,
    color: "bg-muted text-muted-foreground",
  }
  const StatusIcon = status.icon

  const projectKey = issue.display_key.split("-")[0]

  return (
    <div className="flex h-[calc(100svh-4.5rem)] flex-col gap-6">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" className="cursor-pointer" onClick={() => router.back()}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <Link
          href={`/${orgSlug}/projects/${projectKey}`}
          className="font-mono text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          {issue.display_key}
        </Link>
      </div>

      <div className="flex-1 overflow-y-auto">
        <h1 className="text-2xl font-semibold mb-4">{issue.title}</h1>

        <div className="flex items-center gap-4 mb-6">
          <Badge className={`${status.color} gap-1.5`}>
            <StatusIcon className="h-3.5 w-3.5" />
            {status.label}
          </Badge>

          {issue.assignee_id && (
            <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              <span>Assigned</span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-6 text-sm text-muted-foreground mb-8">
          <div className="flex items-center gap-1.5">
            <Calendar className="h-4 w-4" />
            <span>
              Created{" "}
              {new Date(issue.created_at).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <Clock className="h-4 w-4" />
            <span>
              Updated{" "}
              {new Date(issue.updated_at).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          </div>
        </div>

        <div className="border-t pt-6">
          <h2 className="text-sm font-medium text-muted-foreground mb-3">
            Description
          </h2>
          {issue.description ? (
            <div className="prose prose-sm dark:prose-invert max-w-none">
              <p className="whitespace-pre-wrap">{issue.description}</p>
            </div>
          ) : (
            <p className="text-muted-foreground text-sm italic">
              No description provided.
            </p>
          )}
        </div>
      </div>
    </div>
  )
}
