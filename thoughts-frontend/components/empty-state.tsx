interface EmptyStateProps {
  message: string
  className?: string
}

export function EmptyState({ message, className }: EmptyStateProps) {
  return (
    <p className={`text-center text-muted-foreground pt-8 ${className ?? ""}`}>
      {message}
    </p>
  )
}
