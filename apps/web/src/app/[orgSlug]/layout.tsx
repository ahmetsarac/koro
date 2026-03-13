import { AppSidebar } from "@/components/app-sidebar"
import { OrgBreadcrumb } from "@/components/org-breadcrumb"
import { ThemeToggle } from "@/components/theme-toggle"
import { Separator } from "@/components/ui/separator"
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"

export default async function OrgLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params

  return (
    <SidebarProvider>
      <AppSidebar orgSlug={orgSlug} />
      <SidebarInset>
        <header className="sticky top-0 z-20 flex h-12 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12 bg-background border-b">
          <div className="flex items-center gap-2 px-6">
            <SidebarTrigger className="-ml-1" />
            <Separator
              orientation="vertical"
              className="mr-2 data-[orientation=vertical]:h-4"
            />
            <OrgBreadcrumb />
          </div>
          <div className="ml-auto px-6">
            <ThemeToggle />
          </div>
        </header>
        <main className="flex flex-1 flex-col gap-6 px-6 py-2">
          {children}
        </main>
      </SidebarInset>
    </SidebarProvider>
  )
}
