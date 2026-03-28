"use client"

import * as React from "react"
import { useRouter } from "next/navigation"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Label } from "@/components/ui/label"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { ChevronsUpDownIcon, PlusIcon, Settings2 } from "lucide-react"
import { slugFromName } from "@/lib/org-slug"
import { clearAllIssuesCaches } from "@/lib/cache/issues-cache"

interface Organization {
  id: string
  name: string
  slug: string
  role: string
  logo: React.ReactNode
}

export function OrganizationSwitcher({
  organizations,
  currentOrgSlug,
}: {
  organizations: Organization[]
  currentOrgSlug: string
}) {
  const router = useRouter()
  const activeOrganization =
    organizations.find((org) => org.slug === currentOrgSlug) || organizations[0]

  const [addOrgOpen, setAddOrgOpen] = React.useState(false)
  const [orgName, setOrgName] = React.useState("")
  const [orgSlugInput, setOrgSlugInput] = React.useState("")
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    if (!addOrgOpen) return
    setOrgName("")
    setOrgSlugInput("")
    setError(null)
    setLoading(false)
  }, [addOrgOpen])

  if (!activeOrganization) {
    return null
  }

  function handleOrgSwitch(org: Organization) {
    if (org.slug !== currentOrgSlug) {
      clearAllIssuesCaches()
    }
    router.push(`/${org.slug}/my-issues`)
  }

  function handleOrgNameChange(value: string) {
    setOrgName(value)
    if (!orgSlugInput || orgSlugInput === slugFromName(orgName)) {
      setOrgSlugInput(slugFromName(value))
    }
  }

  async function handleCreateOrg(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setLoading(true)
    const name = orgName.trim() || "My Organization"
    const slug =
      orgSlugInput.trim() || slugFromName(orgName) || "my-org"
    try {
      const res = await fetch("/api/orgs", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({ name, slug }),
      })
      const data = (await res.json().catch(() => ({}))) as {
        org_id?: string
        slug?: string
        message?: string
      }
      if (!res.ok) {
        throw new Error(data.message ?? "Could not create organization.")
      }
      const newSlug = data.slug ?? slug
      setAddOrgOpen(false)
      clearAllIssuesCaches()
      router.push(`/${newSlug}/my-issues`)
      router.refresh()
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Could not create organization.",
      )
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                {activeOrganization.logo}
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">
                  {activeOrganization.name}
                </span>
                <span className="truncate text-xs capitalize">
                  {activeOrganization.role.replaceAll("_", " ")}
                </span>
              </div>
              <ChevronsUpDownIcon className="ml-auto" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
            align="start"
            side={"bottom"}
            sideOffset={4}
          >
            <DropdownMenuLabel className="text-xs text-muted-foreground">
              Organizations
            </DropdownMenuLabel>
            {organizations.map((organization, index) => (
              <DropdownMenuItem
                key={organization.slug}
                onClick={() => handleOrgSwitch(organization)}
                className="gap-2 p-2"
              >
                <div className="flex size-6 items-center justify-center rounded-md border">
                  {organization.logo}
                </div>
                {organization.name}
                <DropdownMenuShortcut>⌘{index + 1}</DropdownMenuShortcut>
              </DropdownMenuItem>
            ))}
            <DropdownMenuSeparator />
            <DropdownMenuItem
              className="gap-2 p-2"
              onClick={() => router.push(`/${currentOrgSlug}/settings`)}
            >
              <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                <Settings2 className="size-4" />
              </div>
              <span className="font-medium">Workspace settings</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              className="gap-2 p-2"
              onSelect={(e) => {
                e.preventDefault()
                setAddOrgOpen(true)
              }}
            >
              <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                <PlusIcon className="size-4" />
              </div>
              <span className="font-medium text-muted-foreground">
                Add organization
              </span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>

      <Dialog open={addOrgOpen} onOpenChange={setAddOrgOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Add organization</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleCreateOrg} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="add-org-name">Name</Label>
              <Input
                id="add-org-name"
                value={orgName}
                onChange={(e) => handleOrgNameChange(e.target.value)}
                placeholder="e.g. Acme Inc."
                autoComplete="organization"
                disabled={loading}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="add-org-slug">URL slug</Label>
              <Input
                id="add-org-slug"
                value={orgSlugInput}
                onChange={(e) => setOrgSlugInput(e.target.value)}
                placeholder="acme"
                autoComplete="off"
                disabled={loading}
              />
              <p className="text-xs text-muted-foreground">
                Used in URLs: /{orgSlugInput.trim() || "your-org"}/projects
              </p>
            </div>
            {error && (
              <p className="text-sm text-destructive">{error}</p>
            )}
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setAddOrgOpen(false)}
                disabled={loading}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={loading}>
                {loading ? "Creating…" : "Create"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}
