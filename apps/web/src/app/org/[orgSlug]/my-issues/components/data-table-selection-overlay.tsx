import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Separator } from "@/components/ui/separator"
import { Grip, X } from "lucide-react"

export function DataTableSelectionOverlay({
    selectedCount,
    onClearSelection,
  }: {
    selectedCount: number
    onClearSelection: () => void
  }) {
    if (selectedCount === 0) return null
  
    return (
      <div className="pointer-events-none absolute inset-x-0 bottom-4 z-20 flex justify-center px-4">
        <div className="pointer-events-auto flex items-center gap-1 rounded-lg border border-white/10 bg-sidebar p-1.5 text-white shadow-[0_4px_8px_rgba(0,0,0,0.28)] backdrop-blur-xl">
          <div className="flex items-center gap-0">
            <div className="flex h-7 items-center rounded-l-md border border-r-0 border-dashed border-sidebar-border bg-sidebar px-2 text-xs font-medium tracking-tight text-foreground">
              {selectedCount} selected
            </div>
            <Button
              size="icon"
              className="size-7 rounded-l-none rounded-r-md border border-dashed border-sidebar-border bg-sidebar text-foreground hover:bg-background/90 hover:text-foreground"
              onClick={onClearSelection}
            >
              <X />
              <span className="sr-only">Clear selection</span>
            </Button>
          </div>
  
          <Separator
            orientation="vertical"
            className="mx-1 h-5 data-vertical:self-auto"
          />
  
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button className="rounded-md border border-sidebar-border bg-sidebar px-3 text-foreground hover:bg-background/90 hover:text-foreground">
                <Grip />
                Actions
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="center"
              side="top"
              sideOffset={12}
              className="min-w-24"
            >
              <DropdownMenuItem>Edit</DropdownMenuItem>
              <DropdownMenuItem>Delete</DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    )
  }
