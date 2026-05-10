"use client"

import { useSidebar } from "@/components/ui/sidebar"

export function MyIssuesLayoutContent({ children }: { children: React.ReactNode }) {
  const { state } = useSidebar()

  const sidebarWidth = state === "expanded" ? "var(--sidebar-width)" : "var(--sidebar-width-icon)"

  return (
    <section className="flex min-h-0 flex-col gap-4 h-[calc(100svh-4.5rem)] w-[calc(100svh-var(--sidebar-width)-3rem)]" style={{ width: `calc(100svw - ${sidebarWidth} - 3rem)` }}>
      {children}
    </section>
  )
}
