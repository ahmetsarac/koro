"use client"

import * as React from "react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"

interface ConfirmDialogConfig {
  title: string
  description: string
  confirmLabel?: string
  confirmVariant?: "default" | "destructive"
}

interface ConfirmDialogState extends ConfirmDialogConfig {
  open: boolean
  resolve: (value: boolean) => void
}

export function useConfirmDialog() {
  const [state, setState] = React.useState<ConfirmDialogState | null>(null)

  const confirm = React.useCallback(
    (config: ConfirmDialogConfig): Promise<boolean> => {
      return new Promise((resolve) => {
        setState({ ...config, open: true, resolve })
      })
    },
    []
  )

  const handleOpenChange = React.useCallback(
    (open: boolean) => {
      if (!open) {
        state?.resolve(false)
        setState(null)
      }
    },
    [state]
  )

  const handleConfirm = React.useCallback(() => {
    state?.resolve(true)
    setState(null)
  }, [state])

  const dialog = state ? (
    <Dialog open={state.open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{state.title}</DialogTitle>
          <DialogDescription>{state.description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={() => handleOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant={state.confirmVariant ?? "default"}
            onClick={handleConfirm}
          >
            {state.confirmLabel ?? "Confirm"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ) : null

  return { confirm, dialog }
}

interface AlertDialogConfig {
  title: string
  description: string
  okLabel?: string
}

interface AlertDialogState extends AlertDialogConfig {
  open: boolean
  resolve: () => void
}

export function useAlertDialog() {
  const [state, setState] = React.useState<AlertDialogState | null>(null)

  const alert = React.useCallback(
    (config: AlertDialogConfig): Promise<void> => {
      return new Promise((resolve) => {
        setState({ ...config, open: true, resolve })
      })
    },
    []
  )

  const handleOpenChange = React.useCallback(
    (open: boolean) => {
      if (!open) {
        state?.resolve()
        setState(null)
      }
    },
    [state]
  )

  const handleOk = React.useCallback(() => {
    state?.resolve()
    setState(null)
  }, [state])

  const dialog = state ? (
    <Dialog open={state.open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{state.title}</DialogTitle>
          <DialogDescription>{state.description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={handleOk}>{state.okLabel ?? "OK"}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ) : null

  return { alert, dialog }
}
