//! API 处理器模块 (MSM8916 ModemManager 版)
//!
//! 包含所有 HTTP API 的处理函数

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::fs;
use std::process::{Command, Output};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::{error, info, warn};
use zbus::Connection;

use crate::{
    iptables::flush_iptables,
    models::*,
    modem_manager::{
        answer_call, apply_roaming_policy, get_airplane_mode, get_band_lock_status,
        get_baseband_restart_progress, get_call_by_path, get_call_settings, get_cell_location,
        get_cells_data, get_data_connection_status, get_device_info_data, get_is_roaming_mm,
        get_network_info_data, get_operators_list, get_radio_mode, get_signal_strength,
        get_sim_info_data, hangup_all_calls, hangup_call, list_apn_contexts, list_current_calls,
        make_call, register_operator_auto, register_operator_manual, restart_baseband,
        scan_operators, send_sms, set_airplane_mode, set_apn_on_bearer, set_band_lock,
        set_call_waiting, set_data_connection, set_radio_mode, start_cell_monitoring,
        stop_cell_monitoring,
    },
    state::AppState,
    utils::{
        format_uptime, get_active_interfaces, read_cpu_info, read_cpu_load_sync, read_disk_info,
        read_interface_stats, read_memory_info, read_network_interfaces, read_system_info,
        read_uptime, sample_cpu_usage,
    },
};

// ============ 基础接口 ============

/// 处理 OPTIONS 请求（CORS 预检）
pub async fn options_handler() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// GET /api/health
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "Service is running",
            "platform": "msm8916",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

// ============ 设备信息 ============

/// GET /api/device
pub async fn get_device_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_device_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DeviceInfoResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ SIM 卡 ============

/// GET /api/sim
pub async fn get_sim_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_sim_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SimInfoResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ 网络信息 ============

/// GET /api/network
pub async fn get_network_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_network_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInfoResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/cells
pub async fn get_cells(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_cells_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CellsResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/cell-monitor/start
pub async fn start_cell_monitor_handler(State(app): State<AppState>) -> impl IntoResponse {
    if app.cell_monitoring_active.swap(true, Ordering::SeqCst) {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor already active",
                json!({}),
            )),
        );
    }

    match start_cell_monitoring().await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor activated",
                json!({}),
            )),
        ),
        Err(e) => {
            app.cell_monitoring_active.store(false, Ordering::SeqCst);
            (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "Failed: {}",
                    e
                ))),
            )
        }
    }
}

/// POST /api/cell-monitor/stop
pub async fn stop_cell_monitor_handler(State(app): State<AppState>) -> impl IntoResponse {
    if !app.cell_monitoring_active.swap(false, Ordering::SeqCst) {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor already inactive",
                json!({}),
            )),
        );
    }

    match stop_cell_monitoring().await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor deactivated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/radio-mode
pub async fn get_radio_mode_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_radio_mode(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RadioModeResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/radio-mode
pub async fn set_radio_mode_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<RadioModeRequest>,
) -> impl IntoResponse {
    match set_radio_mode(&conn, payload.mode).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Radio mode updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/band-lock
pub async fn get_band_lock_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_band_lock_status(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<BandLockStatus>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/band-lock
pub async fn set_band_lock_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<BandLockRequest>,
) -> impl IntoResponse {
    match set_band_lock(&conn, &payload).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Band selection updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/location/cell-info
pub async fn get_cell_location_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_cell_location(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CellLocationResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/network/operators
pub async fn get_network_operators(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_operators_list(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<OperatorListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/network/operators/scan
pub async fn scan_network_operators(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match scan_operators(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<OperatorListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/network/register-manual
pub async fn register_network_manual(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<ManualRegisterRequest>,
) -> impl IntoResponse {
    match register_operator_manual(&conn, &payload.mccmnc).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Registration started",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/network/register-auto
pub async fn register_network_auto(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match register_operator_auto(&conn).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Auto registration started",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/apn
pub async fn get_apn_list_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match list_apn_contexts(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<ApnListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/apn
pub async fn set_apn_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<SetApnRequest>,
) -> impl IntoResponse {
    match set_apn_on_bearer(&conn, &payload).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("APN updated", json!({}))),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/cell-lock
pub async fn get_cell_lock_status_handler(State(app): State<AppState>) -> impl IntoResponse {
    let store = app.cell_lock.lock().await;
    let data = store.status();
    drop(store);
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", data)),
    )
}

/// POST /api/cell-lock
pub async fn set_cell_lock_handler(
    State(app): State<AppState>,
    Json(payload): Json<CellLockRequest>,
) -> impl IntoResponse {
    let mut store = app.cell_lock.lock().await;
    match store.apply(&payload) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "OK",
                CellLockResult { success: true },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CellLockResult>::error(e)),
        ),
    }
}

/// POST /api/cell-lock/unlock-all
pub async fn unlock_all_cells_handler(State(app): State<AppState>) -> impl IntoResponse {
    let mut store = app.cell_lock.lock().await;
    store.unlock_all();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Unlocked",
            CellLockResult { success: true },
        )),
    )
}

/// GET /api/network/interfaces
pub async fn get_network_interfaces_info() -> impl IntoResponse {
    match read_network_interfaces() {
        Ok(interfaces) => {
            let total_count = interfaces.len();
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Success",
                    NetworkInterfacesResponse {
                        interfaces,
                        total_count,
                    },
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInterfacesResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/network/signal-strength
pub async fn get_signal_strength_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_signal_strength(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SignalStrengthResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ 数据连接 ============

/// GET /api/data
pub async fn get_data_status(State(app): State<AppState>) -> impl IntoResponse {
    if app.data_user_disabled.load(Ordering::SeqCst) {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                DataConnectionResponse { active: false },
            )),
        );
    }

    match get_data_connection_status(&app.dbus_conn).await {
        Ok(active) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                DataConnectionResponse { active },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DataConnectionResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/data
pub async fn set_data_status(
    State(app): State<AppState>,
    Json(payload): Json<DataConnectionRequest>,
) -> impl IntoResponse {
    let allow_roaming = app.config_manager.get_roaming_allowed();
    let _ = flush_iptables().await;
    match set_data_connection(&app.dbus_conn, payload.active, allow_roaming).await {
        Ok(_) => {
            if let Err(err) = app.config_manager.set_data_enabled(payload.active) {
                return (
                    StatusCode::OK,
                    Json(ApiResponse::<DataConnectionResponse>::error(format!(
                        "Failed to save data switch state: {}",
                        err
                    ))),
                );
            }
            app.data_user_disabled
                .store(!payload.active, Ordering::SeqCst);
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Data connection updated",
                    DataConnectionResponse {
                        active: payload.active,
                    },
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DataConnectionResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn restart_baseband_handler(State(app): State<AppState>) -> impl IntoResponse {
    let auto_connect_data = !app.data_user_disabled.load(Ordering::SeqCst);
    let allow_roaming = app.config_manager.get_roaming_allowed();
    match restart_baseband(&app.dbus_conn, auto_connect_data, allow_roaming).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Baseband restarted",
                data,
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<BasebandRestartResponse>::error(format!(
                "重启基带失败：{}",
                e
            ))),
        ),
    }
}

pub async fn get_baseband_restart_status_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Success",
            get_baseband_restart_progress(),
        )),
    )
}

/// GET /api/roaming
pub async fn get_roaming_status_handler(State(app): State<AppState>) -> impl IntoResponse {
    let roaming_allowed = app.config_manager.get_roaming_allowed();
    match get_is_roaming_mm(&app.dbus_conn).await {
        Ok(is_roaming) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                RoamingResponse {
                    roaming_allowed,
                    is_roaming,
                },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RoamingResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/roaming
pub async fn set_roaming_status_handler(
    State(app): State<AppState>,
    Json(payload): Json<RoamingRequest>,
) -> impl IntoResponse {
    match apply_roaming_policy(&app.dbus_conn, &app.config_manager, payload.allowed).await {
        Ok(_) => {
            let roaming_allowed = app.config_manager.get_roaming_allowed();
            match get_is_roaming_mm(&app.dbus_conn).await {
                Ok(is_roaming) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_with_message(
                        "Success",
                        RoamingResponse {
                            roaming_allowed,
                            is_roaming,
                        },
                    )),
                ),
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::<RoamingResponse>::error(format!(
                        "Failed: {}",
                        e
                    ))),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RoamingResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/airplane-mode
pub async fn set_airplane_mode_handler(
    State(app): State<AppState>,
    Json(payload): Json<AirplaneModeRequest>,
) -> impl IntoResponse {
    if payload.enabled {
        app.airplane_mode_requested.store(true, Ordering::SeqCst);
    }

    match set_airplane_mode(&app.dbus_conn, payload.enabled).await {
        Ok(_) => {
            app.airplane_mode_requested
                .store(payload.enabled, Ordering::SeqCst);
            match get_airplane_mode(&app.dbus_conn).await {
                Ok(status) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_with_message(
                        if payload.enabled {
                            "Airplane mode enabled"
                        } else {
                            "Airplane mode disabled"
                        },
                        status,
                    )),
                ),
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                        "Failed: {}",
                        e
                    ))),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/airplane-mode
pub async fn get_airplane_mode_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_airplane_mode(&conn).await {
        Ok(status) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", status)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ 短信功能 ============

use crate::db::Database;

/// POST /api/sms/send
pub async fn send_sms_handler(
    State(conn): State<Arc<Connection>>,
    State(db): State<Arc<Database>>,
    Json(payload): Json<SendSmsRequest>,
) -> impl IntoResponse {
    match send_sms(&conn, &payload.phone_number, &payload.content).await {
        Ok(path) => {
            // 存入数据库
            let _ = db.insert_sms(
                "outgoing",
                &payload.phone_number,
                &payload.content,
                "sent",
                None,
            );
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "SMS sent",
                    json!({ "path": path }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to send SMS: {}",
                e
            ))),
        ),
    }
}

/// GET /api/sms/list
pub async fn get_sms_list_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<SmsListRequest>,
) -> (StatusCode, Json<ApiResponse<SmsListResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    let offset = if params.offset >= 0 { params.offset } else { 0 };
    let direction = params
        .direction
        .as_deref()
        .filter(|value| matches!(*value, "incoming" | "outgoing"));

    match db.get_sms_messages(limit, offset, direction) {
        Ok(messages) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                SmsListResponse { messages },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SmsListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/sms/conversation
pub async fn get_sms_conversation_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<SmsConversationRequest>,
) -> (StatusCode, Json<ApiResponse<SmsListResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    match db.get_sms_conversation(&params.phone_number, limit) {
        Ok(messages) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                SmsListResponse { messages },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SmsListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/sms/stats
pub async fn get_sms_stats_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<SmsStatsResponse>>) {
    match db.get_sms_stats() {
        Ok(stats) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", stats)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SmsStatsResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/sms/clear
pub async fn clear_sms_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.clear_all_sms() {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "All SMS cleared",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// DELETE /api/sms/message/{id}
pub async fn delete_sms_message_handler(
    State(db): State<Arc<Database>>,
    Path(id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.delete_sms(id) {
        Ok(deleted) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "SMS deleted",
                json!({ "deleted": deleted }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// DELETE /api/sms/conversation/{phone_number}
pub async fn delete_sms_conversation_handler(
    State(db): State<Arc<Database>>,
    Path(phone_number): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.delete_sms_conversation(&phone_number) {
        Ok(deleted) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "SMS conversation deleted",
                json!({ "deleted": deleted }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// POST /api/sms/batch-delete
pub async fn delete_sms_batch_handler(
    State(db): State<Arc<Database>>,
    Json(payload): Json<SmsBatchDeleteRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    if payload.ids.is_empty() && payload.phone_numbers.is_empty() {
        return (StatusCode::OK, Json(ApiResponse::error("No SMS selected")));
    }

    match db.delete_sms_batch(&payload.ids, &payload.phone_numbers) {
        Ok(deleted) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "SMS batch deleted",
                json!({ "deleted": deleted }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

// ============ 系统信息 ============

/// 读取温度传感器数据
// ============ 电话功能 ============

async fn track_call_start(
    app: &AppState,
    path: &str,
    direction: &str,
    phone_number: &str,
    answered: bool,
) {
    if let Ok(id) = app.database.insert_call(direction, phone_number, answered) {
        let mut active = app.active_calls.lock().await;
        active.insert(
            path.to_string(),
            crate::state::ActiveCallRecord {
                id,
                answered_at: answered.then(std::time::Instant::now),
                answered,
            },
        );
    }
}

async fn mark_tracked_call_answered(app: &AppState, path: &str) {
    let mut active = app.active_calls.lock().await;
    if let Some(record) = active.get_mut(path) {
        record.answered = true;
        if record.answered_at.is_none() {
            record.answered_at = Some(std::time::Instant::now());
        }
    }
}

async fn finish_tracked_call(app: &AppState, path: &str, answered_now: bool) {
    let mut record = {
        let mut active = app.active_calls.lock().await;
        active.remove(path)
    };
    if let Some(ref mut record) = record {
        if answered_now && record.answered_at.is_none() {
            record.answered_at = Some(std::time::Instant::now());
        }
        let duration = record
            .answered_at
            .map(|at| at.elapsed().as_secs() as i64)
            .unwrap_or(0);
        let _ = app
            .database
            .update_call_end(record.id, duration, record.answered || answered_now);
    }
}

pub async fn get_calls_handler(State(app): State<AppState>) -> impl IntoResponse {
    match list_current_calls(&app.dbus_conn).await {
        Ok(data) => {
            for call in &data.calls {
                if matches!(call.state.as_str(), "active" | "held") {
                    mark_tracked_call_answered(&app, &call.path).await;
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Success", data)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CallListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn dial_call_handler(
    State(app): State<AppState>,
    Json(payload): Json<MakeCallRequest>,
) -> impl IntoResponse {
    let phone_number = payload.phone_number.trim().to_string();
    if phone_number.is_empty() {
        return (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(
                "Phone number is required",
            )),
        );
    }
    match make_call(&app.dbus_conn, &phone_number).await {
        Ok(path) => {
            track_call_start(&app, &path, "outgoing", &phone_number, false).await;
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Call started",
                    json!({ "path": path }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to dial: {}",
                e
            ))),
        ),
    }
}

pub async fn hangup_call_handler(
    State(app): State<AppState>,
    Json(payload): Json<HangupCallRequest>,
) -> impl IntoResponse {
    let before = get_call_by_path(&app.dbus_conn, &payload.path).await.ok();
    match hangup_call(&app.dbus_conn, &payload.path).await {
        Ok(()) => {
            let answered = before
                .as_ref()
                .map(|call| call.state == "active" || call.state == "held")
                .unwrap_or(false);
            finish_tracked_call(&app, &payload.path, answered).await;
            if let Some(call) = before {
                if call.direction == "incoming"
                    && matches!(call.state.as_str(), "incoming" | "waiting")
                {
                    let _ = app
                        .database
                        .insert_call("missed", &call.phone_number, false);
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Call hung up", json!({}))),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to hang up: {}",
                e
            ))),
        ),
    }
}

pub async fn hangup_all_calls_handler(State(app): State<AppState>) -> impl IntoResponse {
    let before = list_current_calls(&app.dbus_conn).await.ok();
    match hangup_all_calls(&app.dbus_conn).await {
        Ok(()) => {
            if let Some(list) = before {
                for call in list.calls {
                    let answered = call.state == "active" || call.state == "held";
                    finish_tracked_call(&app, &call.path, answered).await;
                    if call.direction == "incoming"
                        && matches!(call.state.as_str(), "incoming" | "waiting")
                    {
                        let _ = app
                            .database
                            .insert_call("missed", &call.phone_number, false);
                    }
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "All calls hung up",
                    json!({}),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to hang up calls: {}",
                e
            ))),
        ),
    }
}

pub async fn answer_call_handler(
    State(app): State<AppState>,
    Json(payload): Json<HangupCallRequest>,
) -> impl IntoResponse {
    let before = get_call_by_path(&app.dbus_conn, &payload.path).await.ok();
    match answer_call(&app.dbus_conn, &payload.path).await {
        Ok(()) => {
            if let Some(call) = before {
                track_call_start(&app, &payload.path, "incoming", &call.phone_number, true).await;
                mark_tracked_call_answered(&app, &payload.path).await;
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Call answered",
                    json!({}),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to answer call: {}",
                e
            ))),
        ),
    }
}

pub async fn get_call_history_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<CallHistoryRequest>,
) -> (StatusCode, Json<ApiResponse<CallHistoryResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    let offset = if params.offset >= 0 { params.offset } else { 0 };
    match (db.get_call_history(limit, offset), db.get_call_stats()) {
        (Ok(records), Ok(stats)) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                CallHistoryResponse { records, stats },
            )),
        ),
        (Err(e), _) | (_, Err(e)) => (
            StatusCode::OK,
            Json(ApiResponse::<CallHistoryResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn delete_call_history_handler(
    State(db): State<Arc<Database>>,
    Path(id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.delete_call(id) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Call record deleted",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

pub async fn clear_call_history_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.clear_all_calls() {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Call history cleared",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

pub async fn get_call_settings_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_call_settings(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CallSettingsResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn set_call_settings_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<SetCallSettingRequest>,
) -> impl IntoResponse {
    if payload.property != "VoiceCallWaiting" {
        return (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only VoiceCallWaiting is supported by ModemManager",
            )),
        );
    }
    let enabled = matches!(payload.value.as_str(), "enabled" | "on" | "true" | "1");
    match set_call_waiting(&conn, enabled).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Call setting updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to update call setting: {}",
                e
            ))),
        ),
    }
}

pub async fn get_call_volume_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<CallVolumeResponse>::error(
            "Call volume control is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn set_call_volume_handler(
    Json(payload): Json<SetCallVolumeRequest>,
) -> impl IntoResponse {
    let _ = (
        payload.speaker_volume,
        payload.microphone_volume,
        payload.muted,
    );
    (
        StatusCode::OK,
        Json(ApiResponse::<CallVolumeResponse>::error(
            "Call volume control is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn get_call_forwarding_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<CallForwardingResponse>::error(
            "Call forwarding is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn set_call_forwarding_handler(
    Json(payload): Json<SetCallForwardingRequest>,
) -> impl IntoResponse {
    let _ = (payload.forward_type, payload.number, payload.timeout);
    (
        StatusCode::OK,
        Json(ApiResponse::<CallForwardingResponse>::error(
            "Call forwarding is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn get_ims_status_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<ImsStatusResponse>::error(
            "IMS status is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn get_voicemail_status_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<VoicemailStatusResponse>::error(
            "Voicemail status is not exposed by ModemManager on this backend",
        )),
    )
}

fn read_temperature_sensors() -> Vec<ThermalZone> {
    use std::fs;
    use std::path::Path;

    let thermal_path = Path::new("/sys/class/thermal");
    let mut sensors = Vec::new();

    if let Ok(entries) = fs::read_dir(thermal_path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if name.starts_with("thermal_zone") {
                let zone_path = entry.path();
                let sensor_type = fs::read_to_string(zone_path.join("type"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let temperature = fs::read_to_string(zone_path.join("temp"))
                    .ok()
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .map(|t| t as f64 / 1000.0)
                    .unwrap_or(0.0);

                sensors.push(ThermalZone {
                    zone: name.to_string(),
                    sensor_type,
                    temperature,
                });
            }
        }
    }
    sensors.sort_by(|a, b| a.zone.cmp(&b.zone));
    sensors
}

/// GET /api/stats
pub async fn get_system_stats() -> impl IntoResponse {
    let result: Result<SystemStatsResponse, String> = async {
        let interfaces =
            get_active_interfaces().map_err(|e| format!("Failed to get interfaces: {}", e))?;

        let mut initial: Vec<(String, u64, u64)> = Vec::new();
        for iface in &interfaces {
            if let Ok((rx, tx)) = read_interface_stats(iface) {
                initial.push((iface.clone(), rx, tx));
            }
        }

        // 并行执行 CPU 采样 (200ms) 和网速采样间隔 (1000ms)，节省 200ms
        let (cpu_usage, _) = tokio::join!(
            async { sample_cpu_usage().await.unwrap_or(0.0) },
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)),
        );

        let mut speed_data = Vec::new();
        let elapsed = 1.0_f64;
        for (interface, rx1, tx1) in &initial {
            if let Ok((rx2, tx2)) = read_interface_stats(interface) {
                let rx_speed = rx2.saturating_sub(*rx1);
                let tx_speed = tx2.saturating_sub(*tx1);
                speed_data.push(NetworkSpeed {
                    interface: interface.clone(),
                    rx_bytes_per_sec: rx_speed,
                    tx_bytes_per_sec: tx_speed,
                    total_rx_bytes: rx2,
                    total_tx_bytes: tx2,
                });
            }
        }

        let (total, available, cached, buffers) = read_memory_info()?;
        let used = total.saturating_sub(available);
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let disk = read_disk_info();
        let mut cpu_load = read_cpu_load_sync().unwrap_or_default();
        cpu_load.load_percent = cpu_usage;
        let (uptime, idle) = read_uptime()?;
        let formatted = format_uptime(uptime);
        let system_info = read_system_info()?;
        let temperature = read_temperature_sensors();

        Ok(SystemStatsResponse {
            network_speed: NetworkSpeedResponse {
                interfaces: speed_data,
                interval_seconds: elapsed,
            },
            memory: MemoryInfo {
                total_bytes: total,
                available_bytes: available,
                used_bytes: used,
                used_percent,
                cached_bytes: cached,
                buffers_bytes: buffers,
            },
            disk,
            cpu_load,
            uptime: UptimeInfo {
                uptime_seconds: uptime,
                idle_seconds: idle,
                uptime_formatted: formatted,
            },
            system_info,
            temperature,
        })
    }
    .await;

    match result {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(msg) => (
            StatusCode::OK,
            Json(ApiResponse::<SystemStatsResponse>::error(msg)),
        ),
    }
}

/// GET /api/stats/cpu
pub async fn get_cpu_info() -> impl IntoResponse {
    match read_cpu_info() {
        Ok(info) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", info)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CpuInfo>::error(format!("Failed: {}", e))),
        ),
    }
}

/// GET /api/connectivity
pub async fn get_connectivity_check() -> (StatusCode, Json<ApiResponse<ConnectivityCheckResponse>>)
{
    // 两个 ping 并行执行，超时从 2s 缩短到 1s
    let (ipv4_result, ipv6_result) = tokio::join!(
        async_ping_host("223.5.5.5", false),
        async_ping_host("2400:3200::1", true),
    );
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Connectivity check completed",
            ConnectivityCheckResponse {
                ipv4: ipv4_result,
                ipv6: ipv6_result,
            },
        )),
    )
}

async fn async_ping_host(target: &str, is_ipv6: bool) -> PingResult {
    let cmd = if is_ipv6 { "ping6" } else { "ping" };
    let output = tokio::process::Command::new(cmd)
        .args(["-c", "1", "-W", "1", target])
        .output()
        .await;
    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let latency = parse_ping_latency(&stdout);
                PingResult {
                    success: true,
                    latency_ms: latency,
                    target: target.to_string(),
                    error: None,
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                PingResult {
                    success: false,
                    latency_ms: None,
                    target: target.to_string(),
                    error: Some(if stderr.is_empty() {
                        "Unreachable".to_string()
                    } else {
                        stderr.trim().to_string()
                    }),
                }
            }
        }
        Err(e) => PingResult {
            success: false,
            latency_ms: None,
            target: target.to_string(),
            error: Some(format!("Failed: {}", e)),
        },
    }
}

fn parse_ping_latency(output: &str) -> Option<f64> {
    for line in output.lines() {
        if let Some(time_pos) = line.find("time=") {
            let after_time = &line[time_pos + 5..];
            let num_str: String = after_time
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(latency) = num_str.parse::<f64>() {
                return Some(latency);
            }
        }
    }
    None
}

/// POST /api/system/reboot
pub async fn system_reboot(Json(payload): Json<SystemRebootRequest>) -> impl IntoResponse {
    let delay = payload.delay_seconds;
    tokio::spawn(async move {
        run_safe_os_reboot_sequence(delay).await;
    });
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            format!("System will perform safe OS reboot in {} seconds", delay),
            json!({ "delay_seconds": delay }),
        )),
    )
}

async fn run_safe_os_reboot_sequence(delay_seconds: u32) {
    if delay_seconds > 0 {
        tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds as u64)).await;
    }

    info!("Starting safe OS reboot sequence");

    run_reboot_prep_command("disable modem radio", "mmcli", &["-m", "0", "-d"], false);
    run_reboot_prep_command(
        "stop ModemManager IPC service",
        "systemctl",
        &["stop", "ModemManager"],
        false,
    );
    run_reboot_prep_command("stop qmi-proxy", "killall", &["qmi-proxy"], true);
    cleanup_modemmanager_runtime_cache();
    run_reboot_prep_command("flush filesystem cache", "sync", &[], false);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    info!("Safe OS reboot preparation complete, executing reboot");
    if let Err(err) = Command::new("reboot").output() {
        error!(error = %err, "Failed to execute reboot command");
    }
}

fn run_reboot_prep_command(label: &str, program: &str, args: &[&str], allow_failure: bool) {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => {
            info!(step = label, "Safe OS reboot step completed");
        }
        Ok(output) => {
            let severity = if allow_failure {
                "optional"
            } else {
                "required"
            };
            warn_reboot_prep_failure(label, program, severity, &output);
        }
        Err(err) if allow_failure => {
            warn!(step = label, command = program, error = %err, "Optional safe OS reboot step failed");
        }
        Err(err) => {
            warn!(step = label, command = program, error = %err, "Safe OS reboot step failed");
        }
    }
}

fn cleanup_modemmanager_runtime_cache() {
    const CACHE_DIR: &str = "/var/lib/ModemManager";

    match fs::read_dir(CACHE_DIR) {
        Ok(entries) => {
            let mut removed = 0usize;
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let result = if path.is_dir() {
                            fs::remove_dir_all(&path)
                        } else {
                            fs::remove_file(&path)
                        };

                        match result {
                            Ok(()) => removed += 1,
                            Err(err) => warn!(
                                path = %path.display(),
                                error = %err,
                                "Failed to remove ModemManager runtime cache entry"
                            ),
                        }
                    }
                    Err(err) => warn!(
                        directory = CACHE_DIR,
                        error = %err,
                        "Failed to read ModemManager runtime cache entry"
                    ),
                }
            }
            info!(
                directory = CACHE_DIR,
                removed, "ModemManager runtime cache cleanup completed"
            );
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            info!(
                directory = CACHE_DIR,
                "ModemManager runtime cache directory does not exist"
            );
        }
        Err(err) => {
            warn!(
                directory = CACHE_DIR,
                error = %err,
                "Failed to open ModemManager runtime cache directory"
            );
        }
    }
}

fn warn_reboot_prep_failure(label: &str, program: &str, severity: &str, output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    warn!(
        step = label,
        command = program,
        severity = severity,
        status = %output.status,
        stderr = %stderr,
        stdout = %stdout,
        "Safe OS reboot step returned non-zero status"
    );
}

// ============ Webhook 配置 ============

pub async fn restart_service_handler() -> impl IntoResponse {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let _ = Command::new("systemctl")
            .args(["restart", "simadmin"])
            .output();
    });
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "SimAdmin service will restart",
            json!({}),
        )),
    )
}

use crate::config::ConfigManager;
use crate::webhook::WebhookSender;

/// GET /api/notifications/config
pub async fn get_notification_config_handler(
    State(config_manager): State<Arc<ConfigManager>>,
) -> (
    StatusCode,
    Json<ApiResponse<crate::config::NotificationConfig>>,
) {
    let config = config_manager.get_notifications();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", config)),
    )
}

/// POST /api/notifications/config
pub async fn set_notification_config_handler(
    State(config_manager): State<Arc<ConfigManager>>,
    Json(notification_config): Json<crate::config::NotificationConfig>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match config_manager.set_notifications(notification_config) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Notification config updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// POST /api/notifications/test/{channel}
pub async fn test_notification_channel_handler(
    Path(channel): Path<crate::config::NotificationChannel>,
    State(webhook_sender): State<Arc<WebhookSender>>,
) -> (
    StatusCode,
    Json<ApiResponse<crate::models::WebhookTestResponse>>,
) {
    match webhook_sender.test_channel(channel).await {
        Ok(message) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Notification test successful",
                WebhookTestResponse {
                    success: true,
                    message,
                },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Notification test failed",
                WebhookTestResponse {
                    success: false,
                    message: e,
                },
            )),
        ),
    }
}

// ============ OTA 更新 ============

/// GET /api/ota/status
pub async fn get_ota_status_handler() -> impl IntoResponse {
    let status = crate::ota::get_ota_status();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", status)),
    )
}

/// POST /api/ota/upload
pub async fn upload_ota_handler(body: axum::body::Bytes) -> impl IntoResponse {
    match crate::ota::handle_ota_upload(&body) {
        Ok(response) => {
            let message = if response.validation.valid {
                "OTA uploaded and validated"
            } else {
                "OTA uploaded but validation failed"
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(message, response)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<crate::models::OtaUploadResponse>::error(
                format!("Failed: {}", e),
            )),
        ),
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct GitHubReleaseAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: String,
    target_commitish: Option<String>,
    body: Option<String>,
    html_url: Option<String>,
    assets: Vec<GitHubReleaseAsset>,
}

async fn fetch_latest_github_release(
    client: &reqwest::Client,
    proxy_prefix: &str,
) -> Result<GitHubRelease, String> {
    const LATEST_RELEASE_API: &str = "https://api.github.com/repos/3899/SimAdmin/releases/latest";

    let mut urls = vec![LATEST_RELEASE_API.to_string()];
    if !proxy_prefix.is_empty() {
        urls.push(format!("{}{}", proxy_prefix, LATEST_RELEASE_API));
    }

    let mut last_error = String::new();
    for url in urls {
        match client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    last_error = format!("GitHub Releases request failed: HTTP {}", status);
                    continue;
                }
                return response
                    .json::<GitHubRelease>()
                    .await
                    .map_err(|e| format!("Failed to parse latest release: {}", e));
            }
            Err(e) => {
                last_error = format!("Failed to request latest release: {}", e);
            }
        }
    }

    if last_error.is_empty() {
        Err("GitHub Releases request failed".to_string())
    } else {
        Err(last_error)
    }
}

fn normalize_proxy_prefix(prefix: Option<String>) -> String {
    let Some(prefix) = prefix else {
        return String::new();
    };
    let prefix = prefix.trim();
    if prefix.is_empty() {
        return String::new();
    }
    if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{}/", prefix)
    }
}

fn is_supported_ota_asset(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".tar.gz") || lower.ends_with(".tgz") || lower.ends_with(".zip")
}

/// POST /api/ota/latest-release
pub async fn get_latest_ota_release_handler(
    Json(req): Json<crate::models::OtaOnlinePrepareRequest>,
) -> impl IntoResponse {
    let result: Result<GitHubRelease, String> = async {
        let proxy_prefix = normalize_proxy_prefix(req.proxy_prefix);
        let client = reqwest::Client::builder()
            .user_agent("SimAdmin OTA updater")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        fetch_latest_github_release(&client, &proxy_prefix).await
    }
    .await;

    match result {
        Ok(release) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", release)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<GitHubRelease>::error(format!(
                "Failed: {}. GitHub may have rate-limited this request; try again later or enable a proxy.",
                e
            ))),
        ),
    }
}

/// POST /api/ota/online-prepare
pub async fn prepare_online_ota_handler(
    Json(req): Json<crate::models::OtaOnlinePrepareRequest>,
) -> impl IntoResponse {
    const MAX_OTA_BYTES: u64 = 50 * 1024 * 1024;

    let result: Result<crate::models::OtaUploadResponse, String> = async {
        let proxy_prefix = normalize_proxy_prefix(req.proxy_prefix);
        let client = reqwest::Client::builder()
            .user_agent("SimAdmin OTA updater")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let release = fetch_latest_github_release(&client, &proxy_prefix).await?;

        let asset = release
            .assets
            .iter()
            .find(|asset| is_supported_ota_asset(&asset.name))
            .ok_or_else(|| "No supported OTA asset found in latest release".to_string())?;

        if asset.size > MAX_OTA_BYTES {
            return Err(format!(
                "OTA asset is too large: {} bytes exceeds {} bytes",
                asset.size, MAX_OTA_BYTES
            ));
        }

        let download_url = format!("{}{}", proxy_prefix, asset.browser_download_url);
        let bytes = client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download OTA asset: {}", e))?
            .error_for_status()
            .map_err(|e| format!("OTA asset download failed: {}", e))?
            .bytes()
            .await
            .map_err(|e| format!("Failed to read OTA asset: {}", e))?;

        if bytes.len() as u64 > MAX_OTA_BYTES {
            return Err(format!(
                "OTA asset is too large: {} bytes exceeds {} bytes",
                bytes.len(),
                MAX_OTA_BYTES
            ));
        }

        crate::ota::handle_ota_upload(&bytes)
    }
    .await;

    match result {
        Ok(response) => {
            let message = if response.validation.valid {
                "Online OTA downloaded and validated"
            } else {
                "Online OTA downloaded but validation failed"
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(message, response)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<crate::models::OtaUploadResponse>::error(
                format!("Failed: {}", e),
            )),
        ),
    }
}

/// POST /api/ota/apply
pub async fn apply_ota_handler(
    Json(req): Json<crate::models::OtaApplyRequest>,
) -> impl IntoResponse {
    match crate::ota::apply_ota_update(req.restart_now) {
        Ok(message) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                &message,
                json!({ "applied": true }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/ota/cancel
pub async fn cancel_ota_handler() -> impl IntoResponse {
    match crate::ota::cancel_pending_update() {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Update cancelled",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}
