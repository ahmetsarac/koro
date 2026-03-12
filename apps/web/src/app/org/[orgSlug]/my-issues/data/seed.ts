import fs from "node:fs"
import { faker } from "@faker-js/faker"

const TASK_COUNT = 5000
const TASK_ID_START = 1000
const STATUSES = ["backlog", "todo", "in progress", "done", "canceled"] as const
const LABELS = ["bug", "feature", "documentation"] as const
const PRIORITIES = ["low", "medium", "high"] as const

const tasks = Array.from({ length: TASK_COUNT }, (_, index) => ({
  id: `TASK-${TASK_ID_START + index}`,
  title: faker.hacker
    .phrase()
    .replace(/^./, (letter: string) => letter.toUpperCase()),
  status: faker.helpers.arrayElement(STATUSES),
  label: faker.helpers.arrayElement(LABELS),
  priority: faker.helpers.arrayElement(PRIORITIES),
}))

fs.writeFileSync(
  new URL("./tasks.json", import.meta.url),
  JSON.stringify(tasks, null, 2)
)

console.log(`Generated ${tasks.length} demo tasks.`)
