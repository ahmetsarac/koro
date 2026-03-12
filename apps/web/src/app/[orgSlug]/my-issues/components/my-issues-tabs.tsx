"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"

export function MyIssuesTabs() {
  const pathname = usePathname()
  const activeTab = pathname.endsWith("/created") ? "created" : "assigned"
  
  const segments = pathname.split("/").filter(Boolean)
  const orgSlug = segments[0] || ""

  return (
    <Tabs value={activeTab}>
      <TabsList className="w-fit">
        <TabsTrigger value="assigned" asChild>
          <Link href={`/${orgSlug}/my-issues/assigned`}>Assigned to me</Link>
        </TabsTrigger>
        <TabsTrigger value="created" asChild>
          <Link href={`/${orgSlug}/my-issues/created`}>Created by me</Link>
        </TabsTrigger>
      </TabsList>
    </Tabs>
  )
}
