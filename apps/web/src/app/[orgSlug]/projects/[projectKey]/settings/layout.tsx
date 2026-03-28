import Link from "next/link"
import { ArrowLeft } from "lucide-react"

import { Button } from "@/components/ui/button"
import { ProjectSettingsNav } from "./project-settings-nav"

export default async function ProjectSettingsLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ orgSlug: string; projectKey: string }>
}) {
  const { orgSlug, projectKey } = await params

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-6">
      <div className="flex shrink-0 flex-col gap-1">
        <Button variant="ghost" size="sm" className="w-fit gap-1 px-2" asChild>
          <Link href={`/${orgSlug}/projects/${projectKey}`}>
            <ArrowLeft className="h-4 w-4" />
            Back to project
          </Link>
        </Button>
        <h1 className="text-2xl font-semibold">Settings</h1>
      </div>

      <div className="flex min-h-0 flex-1 gap-8">
        <ProjectSettingsNav orgSlug={orgSlug} projectKey={projectKey} />
        <div className="min-w-0 flex-1">{children}</div>
      </div>
    </div>
  )
}
