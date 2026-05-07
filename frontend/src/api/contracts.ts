export interface ApiResponse<T> {
  status: string
  message: string
  data?: T
}

export interface DeviceInfo {
  imei: string
  manufacturer: string
  model: string
  revision?: string
  online: boolean
  powered: boolean
}

export interface SimInfo {
  present: boolean
  iccid: string
  imsi: string
  phone_numbers: string[]
  sms_center: string
  mcc: string
  mnc: string
}

export interface NetworkInfo {
  operator_name: string
  registration_status: string
  technology_preference: string
  signal_strength: number
  mcc?: string
  mnc?: string
}

export interface ServingCell {
  tech: string
  cell_id: number
  tac: number
}

export interface CellInfo {
  is_serving: boolean
  tech: string
  cell_id?: number
  band: string
  arfcn: string
  pci: string
  rsrp: string
  rsrq: string
  sinr: string
  earfcn?: string
  nrarfcn?: string
  type?: string
  ssb_rsrp?: string
  ssb_rsrq?: string
  ssb_sinr?: string
}

export interface CellsResponse {
  serving_cell: ServingCell
  cells: CellInfo[]
}

export interface QosInfo {
  qci: number
  dl_speed: number
  ul_speed: number
  raw_response?: string
  /** When set, dl/ul are estimated from WWAN netdev byte counters (not 3GPP QCI/AMBR). */
  source?: 'interface'
}

export interface ThermalZone {
  zone: string
  type: string
  temperature: number
}

export interface DataConnectionStatus {
  active: boolean
}

export interface DataConnectionRequest {
  active: boolean
}

export interface RoamingResponse {
  roaming_allowed: boolean
  is_roaming: boolean
}

export interface RoamingRequest {
  allowed: boolean
}

export interface AirplaneModeRequest {
  enabled: boolean
}

export interface AirplaneModeResponse {
  enabled: boolean
  powered: boolean
  online: boolean
}

export interface BasebandRestartStep {
  step: string
  status: string
  detail?: string
}

export interface BasebandRestartResponse {
  steps: BasebandRestartStep[]
  running: boolean
  current_registration?: string
}

export interface NetworkSpeed {
  interface: string
  rx_bytes_per_sec: number
  tx_bytes_per_sec: number
  total_rx_bytes: number
  total_tx_bytes: number
}

export interface NetworkSpeedResponse {
  interfaces: NetworkSpeed[]
  interval_seconds: number
}

export interface MemoryInfo {
  total_bytes: number
  available_bytes: number
  used_bytes: number
  used_percent: number
  cached_bytes: number
  buffers_bytes: number
}

export interface UptimeInfo {
  uptime_seconds: number
  idle_seconds: number
  uptime_formatted: string
}

export interface SystemInfo {
  sysname: string
  nodename: string
  release: string
  version: string
  machine: string
  domainname?: string
  full_info: string
}

export interface DiskInfo {
  mount_point: string
  fs_type: string
  total_bytes: number
  used_bytes: number
  available_bytes: number
  used_percent: number
}

export interface CpuLoadInfo {
  load_1min: number
  load_5min: number
  load_15min: number
  core_count: number
  load_percent: number
}

export interface SystemStatsResponse {
  network_speed: NetworkSpeedResponse
  memory: MemoryInfo
  disk: DiskInfo[]
  cpu_load: CpuLoadInfo
  uptime: UptimeInfo
  system_info: SystemInfo
  temperature: ThermalZone[]
}

export interface IpAddress {
  address: string
  prefix_len: number
  ip_type: string
  scope: string
}

export interface NetworkInterfaceInfo {
  name: string
  status: string
  mac_address?: string
  mtu: number
  ip_addresses: IpAddress[]
  rx_bytes: number
  tx_bytes: number
  rx_packets: number
  tx_packets: number
  rx_errors: number
  tx_errors: number
}

export interface NetworkInterfacesResponse {
  interfaces: NetworkInterfaceInfo[]
  total_count: number
}

export type RadioMode = 'auto' | 'lte' | 'nr'

export interface RadioModeResponse {
  mode: string
  technology_preference: string
  supported_modes: string[]
}

export interface BandLockStatus {
  locked: boolean
  lte_fdd_bands: number[]
  lte_tdd_bands: number[]
  nr_fdd_bands: number[]
  nr_tdd_bands: number[]
}

export interface BandLockRequest {
  lte_fdd_bands: number[]
  lte_tdd_bands: number[]
  nr_fdd_bands: number[]
  nr_tdd_bands: number[]
}

export interface CellLockRatStatus {
  rat: number
  rat_name: string
  enabled: boolean
  lock_type: number
  pci: number | null
  arfcn: number | null
}

export interface CellLockStatusResponse {
  rat_status?: CellLockRatStatus[]
  any_locked: boolean
}

export interface CellLockRequest {
  rat: number
  enable: boolean
  lock_type?: number
  pci?: number
  arfcn?: number
}

export interface CellLockResult {
  locked?: boolean
  tech?: string
  arfcn?: number
  pci?: number
  success?: boolean
  steps?: string[]
  raw_response?: string
}

export interface SmsMessage {
  id: number
  direction: string
  phone_number: string
  content: string
  timestamp: string
  status: string
  pdu?: string
}

export interface SmsListRequest {
  limit?: number
  offset?: number
  direction?: 'incoming' | 'outgoing'
}

export interface SmsConversationRequest {
  phone_number: string
  limit?: number
}

export interface SmsStats {
  total: number
  incoming: number
  outgoing: number
}

export interface CallInfo {
  path: string
  phone_number: string
  state: string
  direction: string
  start_time?: string
}

export interface CallListResponse {
  calls: CallInfo[]
}

export interface CallRecord {
  id: number
  direction: string
  phone_number: string
  duration: number
  start_time: string
  end_time?: string
  answered: boolean
}

export interface CallStats {
  total: number
  incoming: number
  outgoing: number
  missed: number
  total_duration: number
}

export interface CallHistoryResponse {
  records: CallRecord[]
  stats: CallStats
}

export interface CallSettingsResponse {
  calling_line_presentation: string
  calling_name_presentation: string
  connected_line_presentation: string
  connected_line_restriction: string
  called_line_presentation: string
  calling_line_restriction: string
  hide_caller_id: string
  voice_call_waiting: string
}

export interface SignalStrengthResponse {
  strength: number
}

export interface CellLocationInfo {
  mcc: string
  mnc: string
  lac: number
  cid: number
  signal_strength: number
  radio_type: string
  arfcn?: number
  pci?: number
  rsrq?: number
  sinr?: number
}

export interface CellLocationResponse {
  available: boolean
  cell_info?: CellLocationInfo
  neighbor_cells: CellLocationInfo[]
  cells?: CellLocationInfo[]
}

export interface OperatorInfo {
  path: string
  name: string
  status: string
  mcc: string
  mnc: string
  technologies: string[]
}

export interface OperatorListResponse {
  operators: OperatorInfo[]
}

export interface ManualRegisterRequest {
  mccmnc: string
}

export interface ApnContext {
  path: string
  name: string
  active: boolean
  apn: string
  protocol: string
  username: string
  password: string
  auth_method: string
  context_type?: string
}

export interface ApnListResponse {
  contexts: ApnContext[]
}

export interface SetApnRequest {
  context_path: string
  apn?: string
  protocol?: string
  username?: string
  password?: string
  auth_method?: string
}

export interface PingResult {
  success: boolean
  latency_ms?: number
  target: string
  error?: string
}

export interface ConnectivityCheckResponse {
  ipv4: PingResult
  ipv6: PingResult
}

export interface WebhookConfig {
  enabled: boolean
  url: string
  forward_sms: boolean
  forward_calls: boolean
  headers: Record<string, string>
  secret: string
  sms_template: string
  call_template: string
}

export type NotificationChannelKey =
  | 'webhook'
  | 'bark'
  | 'wecom_app'
  | 'wecom_robot'
  | 'dingtalk_robot'
  | 'dingtalk_app'
  | 'feishu_robot'
  | 'telegram'

export interface MessageChannelConfig {
  enabled: boolean
  forward_sms: boolean
  forward_calls: boolean
  sms_template: string
  call_template: string
}

export interface BarkConfig extends MessageChannelConfig {
  server_url: string
  device_key: string
  title_template: string
  group: string
  sound: string
  level: string
  icon: string
  click_url: string
  copy: string
  auto_copy: boolean
  save_history: boolean
}

export interface WecomAppConfig extends MessageChannelConfig {
  corp_id: string
  agent_id: string
  secret: string
  to_user: string
  to_party: string
  to_tag: string
  safe: boolean
}

export interface WecomRobotConfig extends MessageChannelConfig {
  webhook_url: string
  key: string
}

export interface DingtalkRobotConfig extends MessageChannelConfig {
  webhook_url: string
  access_token: string
  secret: string
  at_mobiles: string
  at_all: boolean
}

export interface DingtalkAppConfig extends MessageChannelConfig {
  app_key: string
  app_secret: string
  robot_code: string
  open_conversation_id: string
  msg_key: string
}

export interface FeishuRobotConfig extends MessageChannelConfig {
  webhook_url: string
  token: string
  secret: string
}

export interface TelegramConfig extends MessageChannelConfig {
  bot_token: string
  chat_id: string
  parse_mode: string
  disable_web_page_preview: boolean
}

export interface NotificationConfig {
  webhook: WebhookConfig
  bark: BarkConfig
  wecom_app: WecomAppConfig
  wecom_robot: WecomRobotConfig
  dingtalk_robot: DingtalkRobotConfig
  dingtalk_app: DingtalkAppConfig
  feishu_robot: FeishuRobotConfig
  telegram: TelegramConfig
}

export const DEFAULT_SMS_TEMPLATE = `{
  "msg_type": "text",
  "content": {
    "text": "📱 短信通知\\n发送方: {{phone_number}}\\n内容: {{content}}\\n时间: {{timestamp}}"
  }
}`

export const DEFAULT_CALL_TEMPLATE = `{
  "msg_type": "text",
  "content": {
    "text": "📞 来电通知\\n号码: {{phone_number}}\\n类型: {{direction}}\\n时间: {{start_time}}\\n时长: {{duration}}秒\\n已接听: {{answered}}"
  }
}`

export const DEFAULT_PLAIN_SMS_TEMPLATE = `📱 短信通知
发送方: {{phone_number}}
内容: {{content}}
时间: {{timestamp}}`

export const DEFAULT_PLAIN_CALL_TEMPLATE = `📞 来电通知
号码: {{phone_number}}
类型: {{direction}}
时间: {{start_time}}
时长: {{duration}}秒
已接听: {{answered}}`

export interface WebhookTestResponse {
  success: boolean
  message: string
}

export interface OtaMeta {
  version: string
  commit: string
  build_time: string
  binary_md5: string
  frontend_md5: string
  arch: string
  min_version?: string
}

export interface OtaValidation {
  valid: boolean
  is_newer: boolean
  binary_md5_match: boolean
  frontend_md5_match: boolean
  arch_match: boolean
  error?: string
}

export interface OtaStatusResponse {
  current_version: string
  current_commit: string
  pending_update: boolean
  pending_meta?: OtaMeta
}

export interface OtaUploadResponse {
  meta: OtaMeta
  validation: OtaValidation
}

export interface OtaOnlinePrepareRequest {
  proxy_prefix?: string
}

export interface OtaReleaseAsset {
  name: string
  size: number
  browser_download_url: string
}

export interface OtaLatestReleaseResponse {
  tag_name: string
  name?: string
  published_at: string
  target_commitish?: string
  body?: string
  html_url?: string
  assets?: OtaReleaseAsset[]
}
