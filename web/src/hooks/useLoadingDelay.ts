import { useState, useEffect } from 'react'

/**
 * 自定义 Hook：延迟展示加载状态
 * @param loading 原始加载状态
 * @param delay 延迟时间（毫秒），默认为 200ms
 * @returns 经过处理的是否应展示加载 UI 的状态
 */
export function useLoadingDelay(loading: boolean, delay: number = 200) {
  const [showLoadingUI, setShowLoadingUI] = useState(false)

  useEffect(() => {
    let timer: NodeJS.Timeout
    if (loading) {
      timer = setTimeout(() => setShowLoadingUI(true), delay)
    } else {
      setShowLoadingUI(false)
    }
    return () => clearTimeout(timer)
  }, [loading, delay])

  return showLoadingUI
}
