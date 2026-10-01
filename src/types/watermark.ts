export type TileMode = 'tile' | 'single'

export type OutputFormat = 'jpeg' | 'png' | 'heic' | 'sameAsInput'

export type GridPosition =
  | 'topLeft'
  | 'topCenter'
  | 'topRight'
  | 'middleLeft'
  | 'center'
  | 'middleRight'
  | 'bottomLeft'
  | 'bottomCenter'
  | 'bottomRight'

export interface ShadowConfig {
  color: string
  blur: number
  offsetX: number
  offsetY: number
}

export interface VisibleWatermark {
  enabled: boolean
  text: string
  mode: TileMode
  angle: number
  opacity: number
  fontSize: number
  fontSizeRatio: number
  lineSpacing: number
  color: string
  strokeColor: string | null
  strokeWidth: number
  shadow: ShadowConfig | null
  position: GridPosition | null
  customXy: [number, number] | null
}

export interface InvisibleWatermark {
  payload: string
  key: string
}

export interface OutputConfig {
  format: OutputFormat
  stripMetadata: boolean
}

export interface WatermarkConfig {
  visible: VisibleWatermark
  invisible: InvisibleWatermark | null
  output: OutputConfig
}
