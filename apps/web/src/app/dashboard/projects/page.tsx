const projects = [
  {
    key: "KORO",
    name: "Core Platform",
    status: "In progress",
  },
  {
    key: "OPS",
    name: "Operations",
    status: "Backlog",
  },
  {
    key: "WEB",
    name: "Web App",
    status: "Todo",
  },
];

export default function DashboardProjectsPage() {
  return (
    <section className="rounded-[2rem] border border-zinc-800 bg-zinc-900 p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm uppercase tracking-[0.25em] text-zinc-500">
            Protected route
          </p>
          <h2 className="mt-2 text-2xl font-semibold">Projects</h2>
        </div>
        <span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-sm text-emerald-300">
          Auth checked
        </span>
      </div>

      <div className="mt-6 space-y-3">
        {projects.map((project) => (
          <div
            key={project.key}
            className="flex items-center justify-between rounded-2xl border border-zinc-800 px-4 py-3"
          >
            <div>
              <p className="font-medium">{project.name}</p>
              <p className="text-sm text-zinc-500">{project.key}</p>
            </div>
            <p className="text-sm text-zinc-300">{project.status}</p>
          </div>
        ))}
      </div>
    </section>
  );
}
