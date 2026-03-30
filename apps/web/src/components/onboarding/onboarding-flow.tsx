"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { slugFromName } from "@/lib/org-slug"

export function OnboardingFlow() {
  const router = useRouter()

  const [orgName, setOrgName] = useState("")
  const [orgSlugInput, setOrgSlugInput] = useState("")

  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleOrgNameChange = (value: string) => {
    setOrgName(value)
    if (!orgSlugInput || orgSlugInput === slugFromName(orgName)) {
      setOrgSlugInput(slugFromName(value))
    }
  }

  const handleCreateOrg = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      const res = await fetch("/api/orgs", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: orgName.trim() || "My Organization",
          slug: orgSlugInput.trim() || slugFromName(orgName) || "my-org",
        }),
      })
      const data = await res.json().catch(() => ({})) as {
        org_id?: string
        slug?: string
        message?: string
      }
      if (!res.ok) {
        throw new Error(data.message ?? "Organizasyon oluşturulamadı")
      }
      const slug = data.slug ?? (orgSlugInput.trim() || slugFromName(orgName))
      router.replace(`/${slug}/projects`)
      router.refresh()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Bir hata oluştu")
    } finally {
      setLoading(false)
    }
  }

  return (
    <Card className="w-full max-w-md border bg-card/95 shadow-sm backdrop-blur">
      <CardHeader>
        <CardTitle className="text-2xl">Organizasyonu oluştur</CardTitle>
        <CardDescription>
          Issue takibi için önce bir organizasyon oluştur. Projeleri daha sonra
          Projects sayfasından ekleyebilirsin.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleCreateOrg} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="org-name">Organizasyon adı</Label>
            <Input
              id="org-name"
              value={orgName}
              onChange={(e) => handleOrgNameChange(e.target.value)}
              placeholder="Örn. Acme Inc."
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="org-slug">URL slug</Label>
            <Input
              id="org-slug"
              value={orgSlugInput}
              onChange={(e) => setOrgSlugInput(e.target.value)}
              placeholder="acme"
            />
            <p className="text-xs text-muted-foreground">
              URL’de kullanılacak: /acme/projects
            </p>
          </div>
          {error && (
            <div className="text-sm text-destructive">{error}</div>
          )}
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "Oluşturuluyor..." : "Organizasyonu oluştur"}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
