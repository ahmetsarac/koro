"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"

import { cn } from "@/lib/utils"

export function ProjectSettingsNav({
  orgSlug,
  projectKey,
}: {
  orgSlug: string
  projectKey: string
}) {
  const pathname = usePathname()
  const base = `/${orgSlug}/projects/${projectKey}/settings`

  const items = [
    { href: base, label: "General", match: (p: string) => p === base || p === `${base}/` },
    {
      href: `${base}/workflow-statuses`,
      label: "Workflow statuses",
      match: (p: string) =>
        p === `${base}/workflow-statuses` || p.startsWith(`${base}/workflow-statuses/`),
    },
  ]

  return (
    <nav className="flex w-48 shrink-0 flex-col gap-0.5" aria-label="Project settings">
      {items.map(({ href, label, match }) => {
        const active = match(pathname)
        return (
          <Link
            key={href}
            href={href}
            className={cn(
              "rounded-md px-3 py-2 text-sm font-medium transition-colors",
              active
                ? "bg-muted font-medium text-foreground"
                : "text-muted-foreground hover:bg-muted/60 hover:text-foreground"
            )}
          >
            {label}
          </Link>
        )
      })}
    </nav>
  )
}
