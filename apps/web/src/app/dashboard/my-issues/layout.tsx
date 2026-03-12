"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"

export default function MyIssuesLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const pathname = usePathname()
  const activeTab = pathname.endsWith("/created") ? "created" : "assigned"

  return (
    <div className="flex h-[calc(100svh-4.5rem)] min-h-0 flex-col">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold tracking-tight">My Issues</h2>
        <p className="text-muted-foreground">View and manage your issues.</p>
      </div>

      <Tabs value={activeTab} className="flex min-h-0 flex-1 flex-col gap-4">
        <TabsList className="w-fit">
          <TabsTrigger value="assigned" asChild>
            <Link href="/dashboard/my-issues/assigned">Assigned to me</Link>
          </TabsTrigger>
          <TabsTrigger value="created" asChild>
            <Link href="/dashboard/my-issues/created">Created by me</Link>
          </TabsTrigger>
        </TabsList>

        {children}
      </Tabs>
    </div>
  )
}
