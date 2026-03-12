import {
  ArrowDown,
  ArrowRight,
  ArrowUp,
  Ban,
  CheckCircle,
  Circle,
  HelpCircle,
  SignalHigh,
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
    label: "Urgent",
    value: "p0",
    icon: SignalHigh,
  },
  {
    label: "High",
    value: "p1",
    icon: ArrowUp,
  },
  {
    label: "Medium",
    value: "p2",
    icon: ArrowRight,
  },
  {
    label: "Low",
    value: "p3",
    icon: ArrowDown,
  },
]
