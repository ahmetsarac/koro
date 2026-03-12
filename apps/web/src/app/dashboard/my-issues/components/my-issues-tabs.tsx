"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"

export function MyIssuesTabs() {
  const pathname = usePathname()
  const activeTab = pathname.endsWith("/created") ? "created" : "assigned"

  return (
    <Tabs value={activeTab}>
      <TabsList className="w-fit">
        <TabsTrigger value="assigned" asChild>
          <Link href="/dashboard/my-issues/assigned">Assigned to me</Link>
        </TabsTrigger>
        <TabsTrigger value="created" asChild>
          <Link href="/dashboard/my-issues/created">Created by me</Link>
        </TabsTrigger>
      </TabsList>
    </Tabs>
  )
}
