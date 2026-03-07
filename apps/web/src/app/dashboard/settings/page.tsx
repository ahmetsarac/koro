export default function SettingsPage() {
  return (
    <section className="max-w-2xl rounded-[2rem] border border-zinc-800 bg-zinc-900 p-6">
      <p className="text-sm uppercase tracking-[0.25em] text-zinc-500">
        Session
      </p>
      <h2 className="mt-2 text-2xl font-semibold">Auth settings</h2>
      <div className="mt-6 space-y-4 text-sm leading-7 text-zinc-400">
        <p>
          Access token cookie olarak tutulur ve dashboard rotalarina girerken
          middleware tarafinda local olarak verify edilir.
        </p>
        <p>
          Access token gecersizse ve refresh token hala valid ise Next.js sadece
          o anda API refresh endpointini cagirip yeni cookie set eder.
        </p>
      </div>
    </section>
  );
}
