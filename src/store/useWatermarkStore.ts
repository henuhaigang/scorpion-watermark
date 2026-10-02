import { create } from 'zustand'
import type { WatermarkConfig } from '../types/watermark'

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v))

/**
 * 把外部来源（预设、未来的配置导入）的参数钳位到当前 UI 允许的范围内。
 *
 * 旧预设里可能存着超出新版范围的值（例如 lineSpacing 曾经允许到 10，
 * 早期版本还要求 >= 1），直接套用会让水印行距松散或错位。
 */
function normalizeConfig(config: WatermarkConfig): WatermarkConfig {
  return {
    ...config,
    visible: {
      ...config.visible,
      angle: clamp(config.visible.angle, -90, 90),
      opacity: clamp(config.visible.opacity, 0, 1),
      fontSizeRatio: clamp(config.visible.fontSizeRatio, 0.5, 5),
      lineSpacing: clamp(config.visible.lineSpacing, 0, 1.5),
    },
  }
}

interface WatermarkState {
  config: WatermarkConfig
  setConfig: (config: WatermarkConfig) => void
  updateVisible: (updates: Partial<WatermarkConfig['visible']>) => void
  updateOutput: (updates: Partial<WatermarkConfig['output']>) => void
}

const defaultConfig: WatermarkConfig = {
  visible: {
    enabled: true,
    text: '机密文件',
    mode: 'tile',
    angle: -45,
    opacity: 0.3,
    fontSize: 12,
    fontSizeRatio: 3.0,
    lineSpacing: 0.5,
    color: '#000000',
    strokeColor: null,
    strokeWidth: 0,
    shadow: null,
    position: null,
    customXy: null,
  },
  invisible: null,
  output: {
    format: 'sameAsInput',
    stripMetadata: true,
  },
}

export const useWatermarkStore = create<WatermarkState>((set) => ({
  config: defaultConfig,
  setConfig: (config) => set({ config: normalizeConfig(config) }),
  updateVisible: (updates) =>
    set((state) => ({
      config: {
        ...state.config,
        visible: { ...state.config.visible, ...updates },
      },
    })),
  updateOutput: (updates) =>
    set((state) => ({
      config: {
        ...state.config,
        output: { ...state.config.output, ...updates },
      },
    })),
}))
