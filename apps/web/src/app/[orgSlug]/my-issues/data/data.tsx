import {
  ArrowDown,
  ArrowRight,
  ArrowUp,
  Ban,
  CheckCircle,
  Circle,
  HelpCircle,
  Flame,
  Timer,
} from "lucide-react"

const categoryRowIcon: Record<string, typeof Circle> = {
  backlog: HelpCircle,
  unstarted: Circle,
  started: Timer,
  completed: CheckCircle,
  canceled: Ban,
}

export function iconForIssueCategory(category: string) {
  return categoryRowIcon[category] ?? Circle
}

export const priorities = [
  {
    label: "Critical",
    value: "critical",
    icon: Flame,
  },
  {
    label: "High",
    value: "high",
    icon: ArrowUp,
  },
  {
    label: "Medium",
    value: "medium",
    icon: ArrowRight,
  },
  {
    label: "Low",
    value: "low",
    icon: ArrowDown,
  },
]
