import * as Primitive from "@radix-ui/react-context-menu";
import type { ComponentProps } from "react";

export const ContextMenu = Primitive.Root;
export const ContextMenuTrigger = Primitive.Trigger;
export const ContextMenuGroup = Primitive.Group;
export const ContextMenuSub = Primitive.Sub;

const contentClass = "z-50 min-w-[min(11rem,var(--radix-context-menu-content-available-width))] max-w-[min(calc(100vw-16px),var(--radix-context-menu-content-available-width))] max-h-[var(--radix-context-menu-content-available-height)] overflow-y-auto rounded-lg border border-border bg-surface p-1 text-[13px] text-foreground shadow-lg outline-none";
const itemClass = "flex min-h-8 cursor-default select-none items-center gap-2 rounded-md px-2 py-1.5 outline-none data-[highlighted]:bg-pressed data-[disabled]:pointer-events-none data-[disabled]:opacity-40 [&_svg]:size-3.5 [&_svg]:shrink-0";

export function ContextMenuContent({ className = "", collisionPadding = 8, ...props }: ComponentProps<typeof Primitive.Content>) {
  return (
    <Primitive.Portal>
      <Primitive.Content collisionPadding={collisionPadding} className={`${contentClass} ${className}`} {...props} />
    </Primitive.Portal>
  );
}

export function ContextMenuItem({ className = "", variant = "default", ...props }: ComponentProps<typeof Primitive.Item> & { variant?: "default" | "danger" }) {
  return <Primitive.Item className={`${itemClass} ${variant === "danger" ? "text-danger data-[highlighted]:bg-danger/10" : ""} ${className}`} {...props} />;
}

export function ContextMenuLabel({ className = "", ...props }: ComponentProps<typeof Primitive.Label>) {
  return <Primitive.Label className={`px-2 py-1.5 text-[11px] font-medium text-muted ${className}`} {...props} />;
}

export function ContextMenuSeparator({ className = "", ...props }: ComponentProps<typeof Primitive.Separator>) {
  return <Primitive.Separator className={`mx-1 my-1 h-px bg-border ${className}`} {...props} />;
}

/** Visual hint only; the caller owns the actual keyboard shortcut. */
export function ContextMenuShortcut({ className = "", ...props }: ComponentProps<"span">) {
  return <span className={`ml-auto pl-5 text-[11px] tracking-wide text-muted ${className}`} {...props} />;
}

export function ContextMenuSubTrigger({ className = "", children, ...props }: ComponentProps<typeof Primitive.SubTrigger>) {
  return (
    <Primitive.SubTrigger className={`${itemClass} data-[state=open]:bg-pressed ${className}`} {...props}>
      {children}
      <svg aria-hidden="true" viewBox="0 0 16 16" className="ml-auto text-muted" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="m6 4 4 4-4 4" /></svg>
    </Primitive.SubTrigger>
  );
}

export function ContextMenuSubContent({ className = "", sideOffset = 4, collisionPadding = 8, ...props }: ComponentProps<typeof Primitive.SubContent>) {
  return (
    <Primitive.Portal>
      <Primitive.SubContent sideOffset={sideOffset} collisionPadding={collisionPadding} className={`${contentClass} ${className}`} {...props} />
    </Primitive.Portal>
  );
}
