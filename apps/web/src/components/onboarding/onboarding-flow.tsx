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
  const [step, setStep] = useState<1 | 2>(1)
  const [orgSlug, setOrgSlug] = useState<string | null>(null)

  const [orgName, setOrgName] = useState("")
  const [orgSlugInput, setOrgSlugInput] = useState("")
  const [projectKey, setProjectKey] = useState("")
  const [projectName, setProjectName] = useState("")
  const [projectDescription, setProjectDescription] = useState("")

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
      setOrgSlug(data.slug ?? null)
      setStep(2)
      setProjectKey("")
      setProjectName("")
    } catch (err) {
      setError(err instanceof Error ? err.message : "Bir hata oluştu")
    } finally {
      setLoading(false)
    }
  }

  const handleCreateProject = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!orgSlug) return
    setError(null)
    setLoading(true)
    try {
      const key = projectKey.trim().toUpperCase() || "PROJ"
      const res = await fetch(`/api/orgs/${orgSlug}/projects`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          project_key: key,
          name: projectName.trim() || "My Project",
          description: projectDescription.trim() || null,
        }),
      })
      const data = await res.json().catch(() => ({})) as { message?: string }
      if (!res.ok) {
        throw new Error(data.message ?? "Proje oluşturulamadı")
      }
      router.replace(`/${orgSlug}/my-issues`)
      router.refresh()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Bir hata oluştu")
    } finally {
      setLoading(false)
    }
  }

  if (step === 1) {
    return (
      <Card className="w-full max-w-md border bg-card/95 shadow-sm backdrop-blur">
        <CardHeader>
          <CardTitle className="text-2xl">Organizasyonu oluştur</CardTitle>
          <CardDescription>
            Issue takibi için önce bir organizasyon oluştur. Daha sonra bu
            organizasyonun altında proje ekleyebilirsin.
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
              {loading ? "Oluşturuluyor..." : "Devam et"}
            </Button>
          </form>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="w-full max-w-md border bg-card/95 shadow-sm backdrop-blur">
      <CardHeader>
        <CardTitle className="text-2xl">İlk projeyi oluştur</CardTitle>
        <CardDescription>
          Issue açabilmek için organizasyonun altında en az bir proje gerekir.
          Proje anahtarı 2–6 karakter (örn. APP, PAY).
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleCreateProject} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="project-key">Proje anahtarı</Label>
            <Input
              id="project-key"
              value={projectKey}
              onChange={(e) =>
                setProjectKey(e.target.value.replace(/[^a-zA-Z0-9]/g, "").slice(0, 6))
              }
              placeholder="APP"
              maxLength={6}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="project-name">Proje adı</Label>
            <Input
              id="project-name"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              placeholder="Örn. Web Uygulaması"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="project-desc">Açıklama (isteğe bağlı)</Label>
            <Input
              id="project-desc"
              value={projectDescription}
              onChange={(e) => setProjectDescription(e.target.value)}
              placeholder="Kısa açıklama"
            />
          </div>
          {error && (
            <div className="text-sm text-destructive">{error}</div>
          )}
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "Oluşturuluyor..." : "Bitir ve başla"}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
