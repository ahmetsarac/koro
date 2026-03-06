const cards = [
  {
    title: "Auth Durumu",
    body: "Bu alan access token middleware kontrolunden gecince render edilir.",
  },
  {
    title: "Refresh Stratejisi",
    body: "Access token biterse Next.js refresh token ile API refresh route'unu sadece gerektiğinde cagirir.",
  },
  {
    title: "Route Koruma",
    body: "Dashboard altindaki tum route'lar middleware ile ayni sekilde korunur.",
  },
];

export default function DashboardPage() {
  return (
    <section className="space-y-6">
      <div className="rounded-[2rem] border border-zinc-800 bg-gradient-to-br from-zinc-900 via-zinc-900 to-zinc-950 p-8">
        <p className="text-sm uppercase tracking-[0.35em] text-zinc-500">
          Secure area
        </p>
        <h2 className="mt-3 text-3xl font-semibold">Korumali dashboard</h2>
        <p className="mt-3 max-w-2xl text-zinc-400">
          Login sonrasi kullanici burada karsilanir. Route degisikliklerinde
          token dogrulamasi Next.js middleware tarafinda yapilir.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        {cards.map((card) => (
          <article
            key={card.title}
            className="rounded-3xl border border-zinc-800 bg-zinc-900 p-5"
          >
            <h3 className="text-lg font-medium">{card.title}</h3>
            <p className="mt-2 text-sm leading-6 text-zinc-400">{card.body}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
