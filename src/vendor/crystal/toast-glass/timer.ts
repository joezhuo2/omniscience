export function createTimer(ms: number, onDone: () => void) {
  let left = ms
  let startedAt = 0
  let handle: ReturnType<typeof setTimeout> | undefined

  return {
    resume() {
      if (handle !== undefined) return
      startedAt = Date.now()
      handle = setTimeout(onDone, left)
    },
    pause() {
      if (handle === undefined) return
      clearTimeout(handle)
      handle = undefined
      left = Math.max(0, left - (Date.now() - startedAt))
    },
  }
}
