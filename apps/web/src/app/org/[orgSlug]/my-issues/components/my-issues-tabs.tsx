"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"

export function MyIssuesTabs() {
  const pathname = usePathname()
  const activeTab = pathname.endsWith("/created") ? "created" : "assigned"
  
  const segments = pathname.split("/")
  const orgIndex = segments.indexOf("org")
  const orgSlug = orgIndex !== -1 ? segments[orgIndex + 1] : ""

  return (
    <Tabs value={activeTab}>
      <TabsList className="w-fit">
        <TabsTrigger value="assigned" asChild>
          <Link href={`/org/${orgSlug}/my-issues/assigned`}>Assigned to me</Link>
        </TabsTrigger>
        <TabsTrigger value="created" asChild>
          <Link href={`/org/${orgSlug}/my-issues/created`}>Created by me</Link>
        </TabsTrigger>
      </TabsList>
    </Tabs>
  )
}
