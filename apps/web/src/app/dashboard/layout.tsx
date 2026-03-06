import Link from "next/link";
import type { ReactNode } from "react";

import { LogoutButton } from "@/components/auth/logout-button";

const navItems = [
  {
    href: "/dashboard",
    label: "Overview",
  },
  {
    href: "/dashboard/projects",
    label: "Projects",
  },
  {
    href: "/dashboard/settings",
    label: "Settings",
  },
];

export default function DashboardLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-50">
      <div className="mx-auto flex min-h-screen max-w-6xl flex-col px-4 py-6 sm:px-6">
        <header className="flex flex-col gap-4 rounded-3xl border border-zinc-800 bg-zinc-900/80 p-5 backdrop-blur sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.3em] text-zinc-500">
              Koro
            </p>
            <h1 className="text-2xl font-semibold">Dashboard</h1>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            {navItems.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="rounded-full border border-zinc-800 px-4 py-2 text-sm text-zinc-300 transition hover:border-zinc-700 hover:bg-zinc-800 hover:text-zinc-50"
              >
                {item.label}
              </Link>
            ))}

            <LogoutButton />
          </div>
        </header>

        <main className="flex-1 py-6">{children}</main>
      </div>
    </div>
  );
}
