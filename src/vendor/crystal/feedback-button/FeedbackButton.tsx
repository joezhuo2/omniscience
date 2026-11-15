import { useEffect, useRef, useState, type ButtonHTMLAttributes, type ReactNode } from 'react'
import styles from './FeedbackButton.module.css'

export type FeedbackState = 'idle' | 'busy' | 'done' | 'error'

export type FeedbackButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onClick'> & {
  action: () => void | Promise<unknown>
  busy?: ReactNode
  done?: ReactNode
  error?: ReactNode
  resetAfter?: number
  onStateChange?: (state: FeedbackState) => void
}

export function FeedbackButton({
  action,
  busy = '…',
  done = 'Done',
  error = 'Failed',
  resetAfter = 1500,
  onStateChange,
  className,
  children,
  disabled,
  ...rest
}: FeedbackButtonProps) {
  const [state, setState] = useState<FeedbackState>('idle')
  const timer = useRef(0)
  const notify = useRef(onStateChange)
  notify.current = onStateChange

  useEffect(() => () => clearTimeout(timer.current), [])

  const change = (next: FeedbackState) => {
    setState(next)
    notify.current?.(next)
  }

  const run = async () => {
    clearTimeout(timer.current)
    change('busy')
    let result: FeedbackState = 'done'
    try {
      await action()
    } catch {
      result = 'error'
    }
    change(result)
    timer.current = window.setTimeout(() => change('idle'), resetAfter)
  }

  const content = { idle: children, busy, done, error }[state]

  return (
    <button
      type="button"
      {...rest}
      className={[styles.root, className].filter(Boolean).join(' ')}
      data-state={state}
      disabled={disabled || state === 'busy'}
      onClick={run}
    >
      <span aria-live="polite" className={styles.live}>
        <span key={state} className={styles.swap}>
          {content}
        </span>
      </span>
    </button>
  )
}
