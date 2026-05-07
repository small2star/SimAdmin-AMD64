//! Shared request and response models for the active SimAdmin backend.

use serde::{Deserialize, Serialize};

use crate::db::{CallRecord, CallStats, SmsMessage, SmsStats};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success_with_message(message: impl Into<String>, data: T) -> Self {
        Self {
            status: "ok".to_string(),
            message: message.into(),
            data: Some(data),
        }
    }
}

impl<T> ApiResponse<T>
where
    T: Default,
{
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
            data: None,
        }
    }
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct ServingCell {
    pub tech: String,
    pub cell_id: u32,
    pub tac: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct CellInfo {
    pub is_serving: bool,
    pub tech: String,
    #[serde(default)]
    pub cell_id: u32,
    pub band: String,
    pub arfcn: String,
    pub pci: String,
    pub rsrp: String,
    pub rsrq: String,
    pub sinr: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub earfcn: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub nrarfcn: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(rename = "type")]
    pub cell_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ssb_rsrp: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ssb_rsrq: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ssb_sinr: String,
}

#[derive(Debug, Default, Serialize)]
pub struct CellsResponse {
    #[serde(default)]
    pub serving_cell: ServingCell,
    pub cells: Vec<CellInfo>,
}

#[derive(Debug, Default, Serialize)]
pub struct DeviceInfoResponse {
    pub imei: String,
    pub manufacturer: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub online: bool,
    pub powered: bool,
}

#[derive(Debug, Deserialize)]
pub struct DataConnectionRequest {
    pub active: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct DataConnectionResponse {
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct RoamingRequest {
    pub allowed: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct RoamingResponse {
    pub roaming_allowed: bool,
    pub is_roaming: bool,
}

#[derive(Debug, Deserialize)]
pub struct AirplaneModeRequest {
    pub enabled: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct AirplaneModeResponse {
    pub enabled: bool,
    pub powered: bool,
    pub online: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct BasebandRestartStep {
    pub step: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct BasebandRestartResponse {
    pub steps: Vec<BasebandRestartStep>,
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_registration: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ThermalZone {
    pub zone: String,
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub temperature: f64,
}

#[derive(Debug, Default, Serialize)]
pub struct SimInfoResponse {
    pub present: bool,
    pub iccid: String,
    pub imsi: String,
    pub phone_numbers: Vec<String>,
    pub sms_center: String,
    pub mcc: String,
    pub mnc: String,
}

#[derive(Debug, Default, Serialize)]
pub struct NetworkInfoResponse {
    pub operator_name: String,
    pub registration_status: String,
    pub technology_preference: String,
    pub signal_strength: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnc: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RadioMode {
    Auto,
    #[serde(rename = "lte")]
    LteOnly,
    #[serde(rename = "nr")]
    NrOnly,
}

#[derive(Debug, Default, Serialize)]
pub struct RadioModeResponse {
    pub mode: String,
    pub technology_preference: String,
    pub supported_modes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RadioModeRequest {
    pub mode: RadioMode,
}

#[derive(Debug, Default, Serialize)]
pub struct BandLockStatus {
    pub locked: bool,
    #[serde(default)]
    pub lte_fdd_bands: Vec<u32>,
    #[serde(default)]
    pub lte_tdd_bands: Vec<u32>,
    #[serde(default)]
    pub nr_fdd_bands: Vec<u32>,
    #[serde(default)]
    pub nr_tdd_bands: Vec<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BandLockRequest {
    #[serde(default)]
    pub lte_fdd_bands: Vec<u32>,
    #[serde(default)]
    pub lte_tdd_bands: Vec<u32>,
    #[serde(default)]
    pub nr_fdd_bands: Vec<u32>,
    #[serde(default)]
    pub nr_tdd_bands: Vec<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CellLockRatStatus {
    pub rat: u8,
    pub rat_name: String,
    pub enabled: bool,
    pub lock_type: u8,
    pub pci: Option<u16>,
    pub arfcn: Option<u32>,
}

#[derive(Debug, Default, Serialize)]
pub struct CellLockStatusResponse {
    pub rat_status: Vec<CellLockRatStatus>,
    pub any_locked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CellLockRequest {
    #[serde(default = "default_nr_rat")]
    pub rat: u8,
    pub enable: bool,
    #[serde(default)]
    pub lock_type: u8,
    #[serde(default)]
    pub pci: Option<u16>,
    #[serde(default)]
    pub arfcn: Option<u32>,
}

fn default_nr_rat() -> u8 {
    16
}

#[derive(Debug, Deserialize)]
pub struct SystemRebootRequest {
    #[serde(default)]
    pub delay_seconds: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct NetworkSpeed {
    pub interface: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct NetworkSpeedResponse {
    pub interfaces: Vec<NetworkSpeed>,
    pub interval_seconds: f64,
}

#[derive(Debug, Default, Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f64,
    pub cached_bytes: u64,
    pub buffers_bytes: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct UptimeInfo {
    pub uptime_seconds: u64,
    pub idle_seconds: u64,
    pub uptime_formatted: String,
}

#[derive(Debug, Default, Serialize)]
pub struct SystemInfo {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub domainname: String,
    pub full_info: String,
}

#[derive(Debug, Default, Serialize)]
pub struct SystemStatsResponse {
    pub network_speed: NetworkSpeedResponse,
    pub memory: MemoryInfo,
    pub disk: Vec<DiskInfo>,
    pub cpu_load: CpuLoadInfo,
    pub uptime: UptimeInfo,
    pub system_info: SystemInfo,
    pub temperature: Vec<ThermalZone>,
}

#[derive(Debug, Default, Serialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PingResult {
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub target: String,
    pub error: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct ConnectivityCheckResponse {
    pub ipv4: PingResult,
    pub ipv6: PingResult,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CpuLoadInfo {
    pub load_1min: f64,
    pub load_5min: f64,
    pub load_15min: f64,
    pub core_count: u32,
    pub load_percent: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CpuCore {
    pub processor: u32,
    pub bogomips: String,
    pub features: Vec<String>,
    pub implementer: String,
    pub architecture: String,
    pub variant: String,
    pub part: String,
    pub revision: String,
}

#[derive(Debug, Default, Serialize)]
pub struct CpuInfo {
    pub core_count: u32,
    pub cores: Vec<CpuCore>,
    pub hardware: String,
    pub serial: String,
    pub model_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct IpAddress {
    pub address: String,
    pub prefix_len: u8,
    pub ip_type: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    pub mtu: u32,
    pub ip_addresses: Vec<IpAddress>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct NetworkInterfacesResponse {
    pub interfaces: Vec<NetworkInterfaceInfo>,
    pub total_count: usize,
}

#[derive(Debug, Default, Serialize)]
pub struct SignalStrengthResponse {
    pub strength: i32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CellLocationInfo {
    pub mcc: String,
    pub mnc: String,
    pub lac: u32,
    pub cid: u32,
    pub signal_strength: i32,
    pub radio_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arfcn: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pci: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsrq: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sinr: Option<f64>,
}

#[derive(Debug, Default, Serialize)]
pub struct CellLocationResponse {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_info: Option<CellLocationInfo>,
    #[serde(default)]
    pub neighbor_cells: Vec<CellLocationInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cells: Option<Vec<CellLocationInfo>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OperatorInfo {
    pub path: String,
    pub name: String,
    pub status: String,
    pub mcc: String,
    pub mnc: String,
    #[serde(default)]
    pub technologies: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct OperatorListResponse {
    pub operators: Vec<OperatorInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ManualRegisterRequest {
    pub mccmnc: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ApnContext {
    pub path: String,
    pub name: String,
    pub active: bool,
    pub apn: String,
    pub protocol: String,
    pub username: String,
    pub password: String,
    pub auth_method: String,
    #[serde(default)]
    pub context_type: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ApnListResponse {
    pub contexts: Vec<ApnContext>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SetApnRequest {
    pub context_path: String,
    pub apn: Option<String>,
    pub protocol: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_method: Option<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct CellLockResult {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct MakeCallRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct CallInfo {
    pub path: String,
    pub phone_number: String,
    pub state: String,
    pub direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct CallListResponse {
    pub calls: Vec<CallInfo>,
}

#[derive(Debug, Deserialize)]
pub struct HangupCallRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct CallHistoryRequest {
    #[serde(default = "default_page_size")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize, Default)]
pub struct CallHistoryResponse {
    pub records: Vec<CallRecord>,
    pub stats: CallStats,
}

#[derive(Debug, Serialize, Default)]
pub struct CallVolumeResponse {
    pub speaker_volume: u8,
    pub microphone_volume: u8,
    pub muted: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetCallVolumeRequest {
    pub speaker_volume: Option<u8>,
    pub microphone_volume: Option<u8>,
    pub muted: Option<bool>,
}

#[derive(Debug, Serialize, Default)]
pub struct CallSettingsResponse {
    pub calling_line_presentation: String,
    pub calling_name_presentation: String,
    pub connected_line_presentation: String,
    pub connected_line_restriction: String,
    pub called_line_presentation: String,
    pub calling_line_restriction: String,
    pub hide_caller_id: String,
    pub voice_call_waiting: String,
}

#[derive(Debug, Deserialize)]
pub struct SetCallSettingRequest {
    pub property: String,
    pub value: String,
}

#[derive(Debug, Serialize, Default)]
pub struct CallForwardingResponse {
    pub voice_unconditional: String,
    pub voice_busy: String,
    pub voice_no_reply: String,
    pub voice_no_reply_timeout: u16,
    pub voice_not_reachable: String,
    pub forwarding_flag_on_sim: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetCallForwardingRequest {
    pub forward_type: String,
    pub number: String,
    pub timeout: Option<u16>,
}

#[derive(Debug, Serialize, Default)]
pub struct ImsStatusResponse {
    pub registered: bool,
    pub voice_capable: bool,
    pub sms_capable: bool,
}

#[derive(Debug, Serialize, Default)]
pub struct VoicemailStatusResponse {
    pub waiting: bool,
    pub message_count: u8,
    pub mailbox_number: String,
}

#[derive(Debug, Deserialize)]
pub struct SendSmsRequest {
    pub phone_number: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SmsListRequest {
    #[serde(default = "default_page_size")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SmsConversationRequest {
    pub phone_number: String,
    #[serde(default = "default_page_size")]
    pub limit: i64,
}

#[derive(Debug, Default, Deserialize)]
pub struct SmsBatchDeleteRequest {
    #[serde(default)]
    pub ids: Vec<i64>,
    #[serde(default)]
    pub phone_numbers: Vec<String>,
}

fn default_page_size() -> i64 {
    50
}

#[derive(Debug, Default, Serialize)]
pub struct SmsListResponse {
    pub messages: Vec<SmsMessage>,
}

pub type SmsStatsResponse = SmsStats;

#[derive(Debug, Serialize)]
pub struct WebhookTestResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OtaMeta {
    pub version: String,
    pub commit: String,
    pub build_time: String,
    pub binary_md5: String,
    pub frontend_md5: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct OtaStatusResponse {
    pub current_version: String,
    pub current_commit: String,
    pub pending_update: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_meta: Option<OtaMeta>,
}

#[derive(Debug, Default, Serialize)]
pub struct OtaUploadResponse {
    pub meta: OtaMeta,
    pub validation: OtaValidation,
}

#[derive(Debug, Default, Deserialize)]
pub struct OtaOnlinePrepareRequest {
    pub proxy_prefix: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct OtaValidation {
    pub valid: bool,
    pub is_newer: bool,
    pub binary_md5_match: bool,
    pub frontend_md5_match: bool,
    pub arch_match: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OtaApplyRequest {
    #[serde(default)]
    pub restart_now: bool,
}
