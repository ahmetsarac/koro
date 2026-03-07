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

export default function ProjectsPage() {
  return (
    <section>
      <h1 className="text-2xl font-semibold">Projects</h1>
    </section>
  );
}
