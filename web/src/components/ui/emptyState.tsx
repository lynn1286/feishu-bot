import { HTMLAttributes, forwardRef } from 'react'
import { twMerge } from 'tailwind-merge'

interface EmptyStateProps extends HTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode
  title: string
  description?: string
  action?: React.ReactNode
}

export const EmptyState = forwardRef<HTMLDivElement, EmptyStateProps>(
  ({ className, icon, title, description, action, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={twMerge(
          'flex flex-col items-center justify-center py-16 px-4 text-center',
          className
        )}
        {...props}
      >
        {icon ? (
          <div className="mb-4 text-slate-300">{icon}</div>
        ) : (
          <div className="mb-4">
            <svg
              className="w-16 h-16 text-slate-300"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={1}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
              />
            </svg>
          </div>
        )}
        <h3 className="text-lg font-medium text-slate-900 mb-1">{title}</h3>
        {description && <p className="text-sm text-slate-500 max-w-sm mb-4">{description}</p>}
        {action && <div className="mt-2">{action}</div>}
      </div>
    )
  }
)

EmptyState.displayName = 'EmptyState'
