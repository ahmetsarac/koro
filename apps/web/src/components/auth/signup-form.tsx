"use client"

import { useState } from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Label } from "@/components/ui/label"

function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }
  return "Kayıt başarısız"
}

export function SignupForm() {
  const router = useRouter()
  const [email, setEmail] = useState("")
  const [name, setName] = useState("")
  const [password, setPassword] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function onSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setLoading(true)

    try {
      const response = await fetch("/api/auth/signup", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify({
          email: email.trim(),
          name: name.trim(),
          password,
        }),
      })

      const data = (await response.json().catch(() => null)) as
        | { message?: string }
        | null

      if (!response.ok) {
        throw new Error(data?.message ?? "Kayıt başarısız")
      }

      router.replace("/onboarding")
      router.refresh()
    } catch (err) {
      setError(getErrorMessage(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30 p-4">
      <Card className="w-full max-w-sm border bg-card/95 shadow-sm backdrop-blur">
        <CardHeader>
          <CardTitle className="text-2xl">Hesap oluştur</CardTitle>
          <CardDescription>
            Koro issue tracker kullanmaya başlamak için kayıt ol.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">Ad</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                autoComplete="name"
                placeholder="Adın Soyadın"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="email">E-posta</Label>
              <Input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                autoComplete="email"
                placeholder="email@example.com"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Şifre</Label>
              <Input
                id="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                type="password"
                autoComplete="new-password"
                placeholder="En az 8 karakter"
                minLength={8}
                required
              />
            </div>

            {error && (
              <div className="text-sm text-destructive">{error}</div>
            )}

            <Button className="w-full" disabled={loading} type="submit">
              {loading ? "Kaydediliyor..." : "Kayıt ol"}
            </Button>

            <p className="text-center text-sm text-muted-foreground">
              Zaten hesabın var mı?{" "}
              <Link href="/login" className="underline hover:text-foreground">
                Giriş yap
              </Link>
            </p>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
