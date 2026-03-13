"use client"

import * as React from "react"
import { useEditor, EditorContent } from "@tiptap/react"
import StarterKit from "@tiptap/starter-kit"
import { cn } from "@/lib/utils"
import { Toolbar, ToolbarGroup } from "@/components/tiptap-ui-primitive/toolbar"
import { MarkButton } from "@/components/tiptap-ui/mark-button"
import { ListButton } from "@/components/tiptap-ui/list-button"
import { BlockquoteButton } from "@/components/tiptap-ui/blockquote-button"

export interface DescriptionEditorProps {
  value: string
  onChange: (html: string) => void
  placeholder?: string
  className?: string
}

export function DescriptionEditor({
  value,
  onChange,
  placeholder = "Description (optional)",
  className,
}: DescriptionEditorProps) {
  const editor = useEditor(
    {
      extensions: [StarterKit],
      content: value || "",
      editorProps: {
        attributes: {
          class: "min-h-[6rem] w-full resize-none rounded-md border-0 bg-transparent px-2 py-2 text-sm outline-none prose prose-sm dark:prose-invert max-w-none",
        },
      },
      onUpdate: ({ editor }) => {
        onChange(editor.getHTML())
      },
      immediatelyRender: false,
    },
    []
  )

  React.useEffect(() => {
    if (!editor) return
    const current = editor.getHTML()
    if (value !== current && (value === "" || value === "<p></p>")) {
      editor.commands.setContent(value || "", false)
    }
  }, [editor, value])

  if (!editor) return null

  return (
    <div
      className={cn(
        "flex min-h-0 flex-col overflow-hidden rounded-md border border-input bg-input/20 text-xs/relaxed transition-colors focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/30 [&_.tiptap]:min-h-[6rem] [&_.tiptap]:outline-none [&_.tiptap_.ProseMirror-empty:first-child::before]:content-[var(--description-placeholder)] [&_.tiptap_.ProseMirror-empty:first-child::before]:float-left [&_.tiptap_.ProseMirror-empty:first-child::before]:pointer-events-none [&_.tiptap_.ProseMirror-empty:first-child::before]:text-muted-foreground [&_.tiptap_p]:my-1 [&_.tiptap_p:first-child]:mt-0 [&_.tiptap_p:last-child]:mb-0 [&_.tiptap_ul]:my-2 [&_.tiptap_ol]:my-2 [&_.tiptap_li]:my-0.5",
        className
      )}
      style={{ "--description-placeholder": `"${placeholder}"` } as React.CSSProperties}
    >
      <Toolbar
        variant="fixed"
        className={cn(
          "shrink-0 rounded-t-md border-b border-input bg-muted/30 px-1 py-0.5",
          "[&_.tiptap-button]:rounded-md [&_.tiptap-button]:transition-colors [&_.tiptap-button]:border-0",
          "[&_.tiptap-button:hover]:!bg-muted [&_.tiptap-button:hover]:!text-foreground [&_.tiptap-button:hover_.tiptap-button-icon]:!text-foreground",
          "[&_.tiptap-button[data-active-state=on]]:!bg-primary/15 [&_.tiptap-button[data-active-state=on]_.tiptap-button-icon]:!text-primary [&_.tiptap-button[data-active-state=on]:hover]:!bg-primary/25"
        )}
      >
        <ToolbarGroup>
          <MarkButton editor={editor} type="bold" showTooltip={false} />
          <MarkButton editor={editor} type="italic" showTooltip={false} />
          <MarkButton editor={editor} type="strike" showTooltip={false} />
        </ToolbarGroup>
        <ToolbarGroup>
          <ListButton editor={editor} type="bulletList" showTooltip={false} />
          <ListButton editor={editor} type="orderedList" showTooltip={false} />
        </ToolbarGroup>
        <ToolbarGroup>
          <BlockquoteButton editor={editor} showTooltip={false} />
        </ToolbarGroup>
      </Toolbar>
      <div className="min-h-0 flex-1 overflow-y-auto">
        <EditorContent editor={editor} />
      </div>
    </div>
  )
}
