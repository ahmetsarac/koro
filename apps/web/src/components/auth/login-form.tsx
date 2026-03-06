"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  return "Login failed";
}

export function LoginForm({ nextPath }: { nextPath: string }) {
  const router = useRouter();
  const [email, setEmail] = useState("ahmet@koro.local");
  const [password, setPassword] = useState("password123");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify({
          email,
          password,
        }),
      });

      if (!response.ok) {
        const data = (await response.json().catch(() => null)) as
          | { message?: string }
          | null;

        throw new Error(data?.message ?? "Login failed");
      }

      router.replace(nextPath);
      router.refresh();
    } catch (error) {
      setError(getErrorMessage(error));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 p-4 text-zinc-50">
      <Card className="w-full max-w-sm border-zinc-800 bg-zinc-900 text-zinc-50">
        <CardHeader>
          <CardTitle>Koro’ya giriş</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-3">
            <div className="space-y-1">
              <label className="text-sm text-zinc-300">Email</label>
              <Input
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                autoComplete="email"
                className="border-zinc-800 bg-zinc-950"
              />
            </div>

            <div className="space-y-1">
              <label className="text-sm text-zinc-300">Password</label>
              <Input
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                type="password"
                autoComplete="current-password"
                className="border-zinc-800 bg-zinc-950"
              />
            </div>

            {error ? (
              <div className="text-sm text-red-500">{error}</div>
            ) : null}

            <div className="text-xs text-zinc-400">
              Demo kullanıcı: <span className="font-medium">{email}</span>
            </div>

            <Button className="w-full" disabled={loading} type="submit">
              {loading ? "Giriş yapılıyor..." : "Giriş"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
