import type * as React from "react"
import { cn } from "@/lib/utils"

export function Card({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>): React.JSX.Element {
  return (
    <div className={cn("rounded-lg border bg-card text-card-foreground", className)} {...props} />
  )
}

export function CardHeader({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>): React.JSX.Element {
  return <div className={cn("flex flex-col space-y-1.5 p-4", className)} {...props} />
}

export function CardTitle({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLHeadingElement>): React.JSX.Element {
  return (
    <h3 className={cn("font-semibold leading-none tracking-normal", className)} {...props}>
      {children}
    </h3>
  )
}

export function CardDescription({
  className,
  ...props
}: React.HTMLAttributes<HTMLParagraphElement>): React.JSX.Element {
  return <p className={cn("text-sm text-muted-foreground", className)} {...props} />
}

export function CardContent({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>): React.JSX.Element {
  return <div className={cn("p-4 pt-0", className)} {...props} />
}
