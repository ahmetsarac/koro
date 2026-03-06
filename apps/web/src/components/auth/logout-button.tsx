"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";

import { Button } from "@/components/ui/button";

export function LogoutButton() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  async function handleLogout() {
    setLoading(true);

    try {
      await fetch("/api/auth/logout", {
        method: "POST",
      });

      router.replace("/login");
      router.refresh();
    } finally {
      setLoading(false);
    }
  }

  return (
    <Button
      type="button"
      variant="outline"
      className="border-zinc-700 bg-transparent text-zinc-50 hover:bg-zinc-800"
      onClick={handleLogout}
      disabled={loading}
    >
      {loading ? "Cikis..." : "Cikis"}
    </Button>
  );
}
