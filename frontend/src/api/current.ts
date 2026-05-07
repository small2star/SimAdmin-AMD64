import type {
  AirplaneModeRequest,
  AirplaneModeResponse,
  ApiResponse,
  ApnListResponse,
  BandLockRequest,
  BandLockStatus,
  BasebandRestartResponse,
  CallHistoryResponse,
  CallListResponse,
  CallSettingsResponse,
  CellLocationResponse,
  CellLockRequest,
  CellLockResult,
  CellLockStatusResponse,
  CellsResponse,
  ConnectivityCheckResponse,
  DataConnectionRequest,
  DataConnectionStatus,
  DeviceInfo,
  ManualRegisterRequest,
  NetworkInfo,
  NetworkInterfacesResponse,
  NotificationChannelKey,
  NotificationConfig,
  OperatorListResponse,
  OtaStatusResponse,
  OtaLatestReleaseResponse,
  OtaOnlinePrepareRequest,
  OtaUploadResponse,
  RadioMode,
  RadioModeResponse,
  RoamingRequest,
  RoamingResponse,
  SetApnRequest,
  SignalStrengthResponse,
  SimInfo,
  SmsMessage,
  SmsConversationRequest,
  SmsListRequest,
  SmsStats,
  SystemStatsResponse,
  WebhookTestResponse,
} from './types'

type SmsListResponse = {
  messages: SmsMessage[]
}

const API_BASE = '/api'

function throwIfApiEnvelopeError(payload: unknown): void {
  if (typeof payload !== 'object' || payload === null) return
  if (!('status' in payload)) return
  const status = (payload as { status: unknown }).status
  const message = (payload as { message?: unknown }).message
  if (status === 'error' && typeof message === 'string') {
    throw new Error(message)
  }
}

async function request<T>(
  url: string,
  options: RequestInit & { returnText?: boolean } = {},
): Promise<T> {
  const { returnText, ...fetchOptions } = options

  const response = await fetch(`${API_BASE}${url}`, {
    headers: {
      'Content-Type': 'application/json',
      ...fetchOptions.headers,
    },
    ...fetchOptions,
  })

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`)
  }

  if (returnText) {
    return (await response.text()) as T
  }

  const json = (await response.json()) as T
  throwIfApiEnvelopeError(json)
  return json
}

class SimAdminCurrentAPI {
  async health() {
    return request<{ status: string; message: string; version: string }>('/health')
  }

  async getDeviceInfo() {
    return request<ApiResponse<DeviceInfo>>('/device')
  }

  async getSimInfo() {
    return request<ApiResponse<SimInfo>>('/sim')
  }

  async getNetworkInfo() {
    return request<ApiResponse<NetworkInfo>>('/network')
  }

  async getCellsInfo() {
    return request<ApiResponse<CellsResponse>>('/cells')
  }

  async startCellMonitor() {
    return request<ApiResponse<Record<string, never>>>('/cell-monitor/start', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async stopCellMonitor() {
    return request<ApiResponse<Record<string, never>>>('/cell-monitor/stop', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async getDataStatus() {
    return request<ApiResponse<DataConnectionStatus>>('/data')
  }

  async setDataStatus(active: boolean) {
    const body: DataConnectionRequest = { active }
    return request<ApiResponse<DataConnectionStatus>>('/data', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  async restartBaseband() {
    return request<ApiResponse<BasebandRestartResponse>>('/baseband/restart', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async getBasebandRestartStatus() {
    return request<ApiResponse<BasebandRestartResponse>>('/baseband/restart/status')
  }

  async restartService() {
    return request<ApiResponse<Record<string, never>>>('/service/restart', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async rebootSystem(delaySeconds = 1) {
    return request<ApiResponse<{ delay_seconds: number }>>('/system/reboot', {
      method: 'POST',
      body: JSON.stringify({ delay_seconds: delaySeconds }),
    })
  }

  async getRoamingStatus() {
    return request<ApiResponse<RoamingResponse>>('/roaming')
  }

  async setRoamingAllowed(allowed: boolean) {
    const body: RoamingRequest = { allowed }
    return request<ApiResponse<RoamingResponse>>('/roaming', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  async getAirplaneMode() {
    return request<ApiResponse<AirplaneModeResponse>>('/airplane-mode')
  }

  async setAirplaneMode(enabled: boolean) {
    const body: AirplaneModeRequest = { enabled }
    return request<ApiResponse<AirplaneModeResponse>>('/airplane-mode', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  async getSystemStats() {
    return request<ApiResponse<SystemStatsResponse>>('/stats')
  }

  async getNetworkInterfaces() {
    return request<ApiResponse<NetworkInterfacesResponse>>('/network/interfaces')
  }

  async getSignalStrength() {
    return request<ApiResponse<SignalStrengthResponse>>('/network/signal-strength')
  }

  async getCellLocationInfo() {
    return request<ApiResponse<CellLocationResponse>>('/location/cell-info')
  }

  async getOperators() {
    return request<ApiResponse<OperatorListResponse>>('/network/operators')
  }

  async scanOperators() {
    return request<ApiResponse<OperatorListResponse>>('/network/operators/scan')
  }

  async registerOperatorManual(mccmnc: string) {
    const body: ManualRegisterRequest = { mccmnc }
    return request<ApiResponse<Record<string, never>>>('/network/register-manual', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  }

  async registerOperatorAuto() {
    return request<ApiResponse<Record<string, never>>>('/network/register-auto', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async getApnList() {
    return request<ApiResponse<ApnListResponse>>('/apn')
  }

  async setApn(config: SetApnRequest) {
    return request<ApiResponse<Record<string, unknown>>>('/apn', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  async getRadioMode() {
    return request<ApiResponse<RadioModeResponse>>('/radio-mode')
  }

  async setRadioMode(mode: RadioMode) {
    return request<ApiResponse<Record<string, never>>>('/radio-mode', {
      method: 'POST',
      body: JSON.stringify({ mode }),
    })
  }

  async getBandLockStatus() {
    return request<ApiResponse<BandLockStatus>>('/band-lock')
  }

  async setBandLock(config: BandLockRequest) {
    return request<ApiResponse<Record<string, never>>>('/band-lock', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  async getCellLockStatus() {
    return request<ApiResponse<CellLockStatusResponse>>('/cell-lock')
  }

  async setCellLock(config: CellLockRequest) {
    return request<ApiResponse<CellLockResult>>('/cell-lock', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  async unlockAllCells() {
    return request<ApiResponse<CellLockResult>>('/cell-lock/unlock-all', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async getConnectivity() {
    return request<ApiResponse<ConnectivityCheckResponse>>('/connectivity')
  }

  async sendSms(phoneNumber: string, content: string) {
    return request<ApiResponse<{ path: string }>>('/sms/send', {
      method: 'POST',
      body: JSON.stringify({ phone_number: phoneNumber, content }),
    })
  }

  async getSmsList(params?: SmsListRequest) {
    const query = new URLSearchParams()
    if (params?.limit) query.append('limit', params.limit.toString())
    if (params?.offset) query.append('offset', params.offset.toString())
    if (params?.direction) query.append('direction', params.direction)
    const queryStr = query.toString() ? `?${query.toString()}` : ''
    return request<ApiResponse<SmsListResponse>>(`/sms/list${queryStr}`)
  }

  async getSmsConversation(params: SmsConversationRequest) {
    const query = new URLSearchParams()
    query.append('phone_number', params.phone_number)
    if (params.limit) query.append('limit', params.limit.toString())
    return request<ApiResponse<SmsListResponse>>(`/sms/conversation?${query.toString()}`)
  }

  async getSmsStats() {
    return request<ApiResponse<SmsStats>>('/sms/stats')
  }

  async clearAllSms() {
    return request<ApiResponse<Record<string, never>>>('/sms/clear', {
      method: 'POST',
    })
  }

  async deleteSmsMessage(id: number) {
    return request<ApiResponse<{ deleted: number }>>(`/sms/message/${id}`, {
      method: 'DELETE',
    })
  }

  async deleteSmsConversation(phoneNumber: string) {
    return request<ApiResponse<{ deleted: number }>>(
      `/sms/conversation/${encodeURIComponent(phoneNumber)}`,
      {
        method: 'DELETE',
      },
    )
  }

  async deleteSmsBatch(payload: { ids?: number[]; phone_numbers?: string[] }) {
    return request<ApiResponse<{ deleted: number }>>('/sms/batch-delete', {
      method: 'POST',
      body: JSON.stringify(payload),
    })
  }

  async getCalls() {
    return request<ApiResponse<CallListResponse>>('/calls')
  }

  async dialCall(phoneNumber: string) {
    return request<ApiResponse<{ path: string }>>('/call/dial', {
      method: 'POST',
      body: JSON.stringify({ phone_number: phoneNumber }),
    })
  }

  async hangupCall(path: string) {
    return request<ApiResponse<Record<string, never>>>('/call/hangup', {
      method: 'POST',
      body: JSON.stringify({ path }),
    })
  }

  async hangupAllCalls() {
    return request<ApiResponse<Record<string, never>>>('/call/hangup-all', {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async answerCall(path: string) {
    return request<ApiResponse<Record<string, never>>>('/call/answer', {
      method: 'POST',
      body: JSON.stringify({ path }),
    })
  }

  async getCallHistory(params?: { limit?: number; offset?: number }) {
    const query = new URLSearchParams()
    if (params?.limit) query.append('limit', params.limit.toString())
    if (params?.offset) query.append('offset', params.offset.toString())
    const queryStr = query.toString() ? `?${query.toString()}` : ''
    return request<ApiResponse<CallHistoryResponse>>(`/call/history${queryStr}`)
  }

  async deleteCallRecord(id: number) {
    return request<ApiResponse<Record<string, never>>>(`/call/history/${id}`, {
      method: 'DELETE',
    })
  }

  async clearCallHistory() {
    return request<ApiResponse<Record<string, never>>>('/call/history/clear', {
      method: 'POST',
    })
  }

  async getCallSettings() {
    return request<ApiResponse<CallSettingsResponse>>('/call/settings')
  }

  async setCallWaiting(enabled: boolean) {
    return request<ApiResponse<Record<string, never>>>('/call/settings', {
      method: 'POST',
      body: JSON.stringify({ property: 'VoiceCallWaiting', value: enabled ? 'enabled' : 'disabled' }),
    })
  }

  async getNotificationConfig() {
    return request<ApiResponse<NotificationConfig>>('/notifications/config')
  }

  async setNotificationConfig(config: NotificationConfig) {
    return request<ApiResponse<Record<string, unknown>>>('/notifications/config', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  async testNotificationChannel(channel: NotificationChannelKey) {
    return request<ApiResponse<WebhookTestResponse>>(`/notifications/test/${channel}`, {
      method: 'POST',
    })
  }

  async getOtaStatus() {
    return request<ApiResponse<OtaStatusResponse>>('/ota/status')
  }

  async uploadOta(file: File) {
    const response = await fetch(`${API_BASE}/ota/upload`, {
      method: 'POST',
      body: file,
      headers: {
        'Content-Type': 'application/octet-stream',
      },
    })

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }

    return response.json() as Promise<ApiResponse<OtaUploadResponse>>
  }

  async prepareOnlineOta(config: OtaOnlinePrepareRequest) {
    return request<ApiResponse<OtaUploadResponse>>('/ota/online-prepare', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  async getLatestOtaRelease(config: OtaOnlinePrepareRequest) {
    return request<ApiResponse<OtaLatestReleaseResponse>>('/ota/latest-release', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  async applyOta(restartNow = false) {
    return request<ApiResponse<{ applied: boolean }>>('/ota/apply', {
      method: 'POST',
      body: JSON.stringify({ restart_now: restartNow }),
    })
  }

  async cancelOta() {
    return request<ApiResponse<Record<string, unknown>>>('/ota/cancel', {
      method: 'POST',
    })
  }
}

export const api = new SimAdminCurrentAPI()

export * from './types'
