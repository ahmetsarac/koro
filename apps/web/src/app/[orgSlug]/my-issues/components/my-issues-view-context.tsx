"use client"

import * as React from "react"

export type MyIssuesView = "list" | "board"

const MyIssuesViewContext = React.createContext<MyIssuesView>("list")

export function MyIssuesViewProvider({
  initialView,
  children,
}: {
  initialView: MyIssuesView
  children: React.ReactNode
}) {
  return (
    <MyIssuesViewContext.Provider value={initialView}>
      {children}
    </MyIssuesViewContext.Provider>
  )
}

export function useMyIssuesInitialView(): MyIssuesView {
  return React.useContext(MyIssuesViewContext)
}
