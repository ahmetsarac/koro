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

export const statuses = [
  {
    value: "backlog",
    label: "Backlog",
    icon: HelpCircle,
  },
  {
    value: "todo",
    label: "Todo",
    icon: Circle,
  },
  {
    value: "in_progress",
    label: "In Progress",
    icon: Timer,
  },
  {
    value: "blocked",
    label: "Blocked",
    icon: Ban,
  },
  {
    value: "done",
    label: "Done",
    icon: CheckCircle,
  },
]

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
