use crate::config::{
    BarkConfig, ConfigManager, DingtalkAppConfig, DingtalkRobotConfig, FeishuRobotConfig,
    MessageChannelConfig, NotificationChannel, NotificationConfig, PushPlusConfig, TelegramConfig,
    WebhookConfig, WecomAppConfig, WecomRobotConfig,
};
use crate::db::{CallRecord, SmsMessage};
use crate::models::DdnsEvent;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{Client, StatusCode};
use ring::hmac;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BEIJING_UTC_OFFSET_SECONDS: i32 = 8 * 60 * 60;
const NOTIFICATION_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Notification sender for all configured notification channels.
pub struct NotificationSender {
    client: Client,
    config_manager: Arc<ConfigManager>,
    wecom_token_cache: tokio::sync::Mutex<HashMap<(String, String), WecomTokenCacheEntry>>,
}

struct WecomTokenCacheEntry {
    token: String,
    refresh_at: Instant,
}

struct WecomTokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

enum WecomMessageError {
    InvalidAccessToken(String),
    Other(String),
}

impl NotificationSender {
    /// Create a new sender.
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            config_manager,
            wecom_token_cache: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    fn get_config(&self) -> NotificationConfig {
        self.config_manager.get_notifications()
    }

    /// Forward an incoming SMS to all enabled channels.
    pub async fn forward_sms(&self, message: &SmsMessage) -> Result<(), String> {
        let config = self.get_config();
        let mut errors = Vec::new();

        for channel in all_channels() {
            if let Err(err) = self
                .send_sms_to_channel(channel, &config, message, false)
                .await
            {
                errors.push(format!("{}: {}", channel.label(), err));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Forward a call record to all enabled channels.
    #[allow(dead_code)]
    pub async fn forward_call(&self, call: &CallRecord) -> Result<(), String> {
        let config = self.get_config();
        let mut errors = Vec::new();

        for channel in all_channels() {
            if let Err(err) = self
                .send_call_to_channel(channel, &config, call, false)
                .await
            {
                errors.push(format!("{}: {}", channel.label(), err));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Forward a DDNS update/failure event to all enabled channels.
    pub async fn forward_ddns_event(&self, event: &DdnsEvent) -> Result<(), String> {
        let config = self.get_config();
        let mut errors = Vec::new();

        for channel in all_channels() {
            if let Err(err) = self.send_ddns_to_channel(channel, &config, event).await {
                errors.push(format!("{}: {}", channel.label(), err));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Test a specific notification channel with a simulated SMS.
    pub async fn test_channel(&self, channel: NotificationChannel) -> Result<String, String> {
        let config = self.get_config();
        let test_message = SmsMessage {
            id: 0,
            direction: "incoming".to_string(),
            phone_number: "+8613800138000".to_string(),
            content: "这是一条测试短信 (Notification Test)".to_string(),
            timestamp: beijing_now_string(),
            status: "received".to_string(),
            pdu: None,
        };

        self.send_sms_to_channel(channel, &config, &test_message, true)
            .await
    }

    async fn send_sms_to_channel(
        &self,
        channel: NotificationChannel,
        config: &NotificationConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        match channel {
            NotificationChannel::Webhook => {
                self.send_webhook_sms(&config.webhook, message, force).await
            }
            NotificationChannel::Bark => self.send_bark_sms(&config.bark, message, force).await,
            NotificationChannel::PushPlus => {
                self.send_pushplus_sms(&config.pushplus, message, force)
                    .await
            }
            NotificationChannel::WecomApp => {
                self.send_wecom_app_sms(&config.wecom_app, message, force)
                    .await
            }
            NotificationChannel::WecomRobot => {
                self.send_wecom_robot_sms(&config.wecom_robot, message, force)
                    .await
            }
            NotificationChannel::DingtalkRobot => {
                self.send_dingtalk_robot_sms(&config.dingtalk_robot, message, force)
                    .await
            }
            NotificationChannel::DingtalkApp => {
                self.send_dingtalk_app_sms(&config.dingtalk_app, message, force)
                    .await
            }
            NotificationChannel::FeishuRobot => {
                self.send_feishu_robot_sms(&config.feishu_robot, message, force)
                    .await
            }
            NotificationChannel::Telegram => {
                self.send_telegram_sms(&config.telegram, message, force)
                    .await
            }
        }
    }

    async fn send_call_to_channel(
        &self,
        channel: NotificationChannel,
        config: &NotificationConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        match channel {
            NotificationChannel::Webhook => {
                self.send_webhook_call(&config.webhook, call, force).await
            }
            NotificationChannel::Bark => self.send_bark_call(&config.bark, call, force).await,
            NotificationChannel::PushPlus => {
                self.send_pushplus_call(&config.pushplus, call, force).await
            }
            NotificationChannel::WecomApp => {
                self.send_wecom_app_call(&config.wecom_app, call, force)
                    .await
            }
            NotificationChannel::WecomRobot => {
                self.send_wecom_robot_call(&config.wecom_robot, call, force)
                    .await
            }
            NotificationChannel::DingtalkRobot => {
                self.send_dingtalk_robot_call(&config.dingtalk_robot, call, force)
                    .await
            }
            NotificationChannel::DingtalkApp => {
                self.send_dingtalk_app_call(&config.dingtalk_app, call, force)
                    .await
            }
            NotificationChannel::FeishuRobot => {
                self.send_feishu_robot_call(&config.feishu_robot, call, force)
                    .await
            }
            NotificationChannel::Telegram => {
                self.send_telegram_call(&config.telegram, call, force).await
            }
        }
    }

    async fn send_ddns_to_channel(
        &self,
        channel: NotificationChannel,
        config: &NotificationConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        match channel {
            NotificationChannel::Webhook => self.send_webhook_ddns(&config.webhook, event).await,
            NotificationChannel::Bark => self.send_bark_ddns(&config.bark, event).await,
            NotificationChannel::PushPlus => self.send_pushplus_ddns(&config.pushplus, event).await,
            NotificationChannel::WecomApp => {
                self.send_wecom_app_ddns(&config.wecom_app, event).await
            }
            NotificationChannel::WecomRobot => {
                self.send_wecom_robot_ddns(&config.wecom_robot, event).await
            }
            NotificationChannel::DingtalkRobot => {
                self.send_dingtalk_robot_ddns(&config.dingtalk_robot, event)
                    .await
            }
            NotificationChannel::DingtalkApp => {
                self.send_dingtalk_app_ddns(&config.dingtalk_app, event)
                    .await
            }
            NotificationChannel::FeishuRobot => {
                self.send_feishu_robot_ddns(&config.feishu_robot, event)
                    .await
            }
            NotificationChannel::Telegram => self.send_telegram_ddns(&config.telegram, event).await,
        }
    }

    async fn send_webhook_sms(
        &self,
        config: &WebhookConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !force && (!config.enabled || !config.forward_sms) {
            return Ok("Webhook skipped".to_string());
        }
        if config.url.trim().is_empty() {
            return Err("Webhook URL is not configured".to_string());
        }

        let payload = render_sms_template(&config.sms_template, message, true);
        self.send_webhook_raw(config, &payload).await
    }

    async fn send_webhook_call(
        &self,
        config: &WebhookConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !force && (!config.enabled || !config.forward_calls) {
            return Ok("Webhook skipped".to_string());
        }
        if config.url.trim().is_empty() {
            return Err("Webhook URL is not configured".to_string());
        }

        let payload = render_call_template(&config.call_template, call, true);
        self.send_webhook_raw(config, &payload).await
    }

    async fn send_webhook_ddns(
        &self,
        config: &WebhookConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !config.enabled || !config.forward_ddns {
            return Ok("Webhook skipped".to_string());
        }
        if config.url.trim().is_empty() {
            return Err("Webhook URL is not configured".to_string());
        }

        let payload = render_ddns_template(&config.ddns_template, event, true);
        self.send_webhook_raw(config, &payload).await
    }

    async fn send_webhook_raw(
        &self,
        config: &WebhookConfig,
        payload: &str,
    ) -> Result<String, String> {
        let mut request = self.client.post(config.url.trim());
        let mut has_content_type = false;

        for (key, value) in &config.headers {
            if key.eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            request = request.header(key, value);
        }

        if !has_content_type {
            request = request.header("Content-Type", "application/json");
        }

        if !config.secret.trim().is_empty() {
            let signature = compute_legacy_signature(config.secret.trim(), payload);
            request = request.header("X-Webhook-Signature", signature);
        }

        let response = request
            .body(payload.to_string())
            .send()
            .await
            .map_err(|e| format!("Failed to send webhook: {}", e))?;
        response_result(
            "Webhook",
            response.status(),
            response.text().await.unwrap_or_default(),
        )
    }

    async fn send_bark_sms(
        &self,
        config: &BarkConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("Bark skipped".to_string());
        }
        if config.device_key.trim().is_empty() {
            return Err("Bark device key is not configured".to_string());
        }

        let title = render_sms_template(&config.title_template, message, false);
        let body = render_sms_template(&config.common.sms_template, message, false);
        self.send_bark_message(config, title, body).await
    }

    async fn send_bark_call(
        &self,
        config: &BarkConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("Bark skipped".to_string());
        }
        if config.device_key.trim().is_empty() {
            return Err("Bark device key is not configured".to_string());
        }

        let title = "SimAdmin 来电通知".to_string();
        let body = render_call_template(&config.common.call_template, call, false);
        self.send_bark_message(config, title, body).await
    }

    async fn send_bark_ddns(
        &self,
        config: &BarkConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("Bark skipped".to_string());
        }
        if config.device_key.trim().is_empty() {
            return Err("Bark device key is not configured".to_string());
        }
        self.send_bark_message(
            config,
            "SimAdmin DDNS 通知".to_string(),
            render_ddns_template(&config.common.ddns_template, event, false),
        )
        .await
    }

    async fn send_bark_message(
        &self,
        config: &BarkConfig,
        title: String,
        body: String,
    ) -> Result<String, String> {
        let url = format!(
            "{}/{}",
            config.server_url.trim().trim_end_matches('/'),
            encode_path_segment(config.device_key.trim())
        );
        let mut payload = Map::new();
        payload.insert("title".to_string(), json!(title));
        payload.insert("body".to_string(), json!(body));
        insert_non_empty(&mut payload, "group", &config.group);
        insert_non_empty(&mut payload, "sound", &config.sound);
        insert_non_empty(&mut payload, "level", &config.level);
        insert_non_empty(&mut payload, "icon", &config.icon);
        insert_non_empty(&mut payload, "url", &config.click_url);
        if config.auto_copy {
            payload.insert("automaticallyCopy".to_string(), json!(1));
            payload.insert(
                "copy".to_string(),
                json!(if config.copy.trim().is_empty() {
                    body.as_str()
                } else {
                    config.copy.trim()
                }),
            );
        }
        payload.insert(
            "isArchive".to_string(),
            json!(if config.save_history { 1 } else { 0 }),
        );

        self.post_json("Bark", &url, Value::Object(payload)).await
    }

    async fn send_pushplus_sms(
        &self,
        config: &PushPlusConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("PushPlus skipped".to_string());
        }

        let title = render_sms_template(&config.title_template, message, false);
        let content = render_sms_template(&config.common.sms_template, message, false);
        self.send_pushplus_message(config, title, content).await
    }

    async fn send_pushplus_call(
        &self,
        config: &PushPlusConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("PushPlus skipped".to_string());
        }

        let content = render_call_template(&config.common.call_template, call, false);
        self.send_pushplus_message(config, "SimAdmin 来电通知".to_string(), content)
            .await
    }

    async fn send_pushplus_ddns(
        &self,
        config: &PushPlusConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("PushPlus skipped".to_string());
        }

        let content = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_pushplus_message(config, "SimAdmin DDNS 通知".to_string(), content)
            .await
    }

    async fn send_pushplus_message(
        &self,
        config: &PushPlusConfig,
        title: String,
        content: String,
    ) -> Result<String, String> {
        if config.token.trim().is_empty() {
            return Err("PushPlus token is not configured".to_string());
        }

        let mut payload = Map::new();
        payload.insert("token".to_string(), json!(config.token.trim()));
        payload.insert("title".to_string(), json!(title));
        payload.insert("content".to_string(), json!(content));
        insert_non_empty(&mut payload, "topic", &config.topic);
        insert_non_empty(&mut payload, "template", &config.template);
        insert_non_empty(&mut payload, "channel", &config.channel);
        insert_non_empty(&mut payload, "option", &config.option);
        insert_non_empty(&mut payload, "callbackUrl", &config.callback_url);

        self.post_json(
            "PushPlus",
            "https://www.pushplus.plus/send",
            Value::Object(payload),
        )
        .await
    }

    async fn send_wecom_app_sms(
        &self,
        config: &WecomAppConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("企业微信应用消息 skipped".to_string());
        }
        let text = render_sms_template(&config.common.sms_template, message, false);
        self.send_wecom_app_text(config, text).await
    }

    async fn send_wecom_app_call(
        &self,
        config: &WecomAppConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("企业微信应用消息 skipped".to_string());
        }
        let text = render_call_template(&config.common.call_template, call, false);
        self.send_wecom_app_text(config, text).await
    }

    async fn send_wecom_app_ddns(
        &self,
        config: &WecomAppConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("企业微信应用消息 skipped".to_string());
        }
        let text = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_wecom_app_text(config, text).await
    }

    async fn send_wecom_app_text(
        &self,
        config: &WecomAppConfig,
        text: String,
    ) -> Result<String, String> {
        if config.corp_id.trim().is_empty()
            || config.secret.trim().is_empty()
            || config.agent_id.trim().is_empty()
        {
            return Err("企业微信 CorpID、AgentID 或 Secret 未配置".to_string());
        }

        let agent_id = config
            .agent_id
            .trim()
            .parse::<i64>()
            .map_err(|_| "企业微信 AgentID 必须为数字".to_string())?;
        let payload = json!({
            "touser": if config.to_user.trim().is_empty() { "@all" } else { config.to_user.trim() },
            "toparty": config.to_party.trim(),
            "totag": config.to_tag.trim(),
            "msgtype": "text",
            "agentid": agent_id,
            "text": { "content": text },
            "safe": if config.safe { 1 } else { 0 },
        });

        self.post_wecom_app_message(config, payload).await
    }

    async fn post_wecom_app_message(
        &self,
        config: &WecomAppConfig,
        payload: Value,
    ) -> Result<String, String> {
        let corp_id = config.corp_id.trim();
        let secret = config.secret.trim();
        let mut retried = false;

        loop {
            let token = self.fetch_wecom_access_token(corp_id, secret).await?;
            match self
                .post_wecom_app_payload(token.as_str(), payload.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(WecomMessageError::InvalidAccessToken(err)) if !retried => {
                    retried = true;
                    self.invalidate_wecom_access_token(corp_id, secret).await;
                    continue;
                }
                Err(WecomMessageError::InvalidAccessToken(err)) => return Err(err),
                Err(WecomMessageError::Other(err)) => return Err(err),
            }
        }
    }

    async fn post_wecom_app_payload(
        &self,
        access_token: &str,
        payload: Value,
    ) -> Result<String, WecomMessageError> {
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
            access_token
        );
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                WecomMessageError::Other(format!("Failed to send 企业微信应用消息 message: {}", e))
            })?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if is_wecom_access_token_error(&body) {
            return Err(WecomMessageError::InvalidAccessToken(
                response_result("企业微信应用消息", status, body).unwrap_or_else(|err| err),
            ));
        }

        response_result("企业微信应用消息", status, body).map_err(WecomMessageError::Other)
    }

    async fn fetch_wecom_access_token(
        &self,
        corp_id: &str,
        secret: &str,
    ) -> Result<String, String> {
        let cache_key = (corp_id.to_string(), secret.to_string());
        let mut cache = self.wecom_token_cache.lock().await;
        if let Some(entry) = cache.get(&cache_key) {
            if Instant::now() < entry.refresh_at {
                return Ok(entry.token.clone());
            }
        }

        let parsed = self.request_wecom_access_token(corp_id, secret).await?;
        let expires_in = parsed.expires_in.unwrap_or(7200).max(1);
        let refresh_after = if expires_in > 600 {
            expires_in - 300
        } else {
            (expires_in / 2).max(1)
        };
        let token = parsed.access_token;
        cache.insert(
            cache_key,
            WecomTokenCacheEntry {
                token: token.clone(),
                refresh_at: Instant::now() + Duration::from_secs(refresh_after),
            },
        );

        Ok(token)
    }

    async fn invalidate_wecom_access_token(&self, corp_id: &str, secret: &str) {
        let mut cache = self.wecom_token_cache.lock().await;
        cache.remove(&(corp_id.to_string(), secret.to_string()));
    }

    async fn request_wecom_access_token(
        &self,
        corp_id: &str,
        secret: &str,
    ) -> Result<WecomTokenResponse, String> {
        #[derive(Debug, Deserialize)]
        struct RawWecomTokenResponse {
            #[serde(default)]
            errcode: i64,
            #[serde(default)]
            errmsg: String,
            #[serde(default)]
            access_token: String,
            #[serde(default)]
            expires_in: Option<u64>,
        }

        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}",
            encode_query_value(corp_id),
            encode_query_value(secret)
        );
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get WeCom access token: {}", e))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("WeCom token request failed ({}): {}", status, body));
        }
        let parsed: RawWecomTokenResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse WeCom token response: {}", e))?;
        if parsed.errcode != 0 {
            return Err(format!(
                "WeCom token error {}: {}",
                parsed.errcode, parsed.errmsg
            ));
        }
        if parsed.access_token.is_empty() {
            return Err("WeCom token response did not include access_token".to_string());
        }
        Ok(WecomTokenResponse {
            access_token: parsed.access_token,
            expires_in: parsed.expires_in,
        })
    }

    async fn send_wecom_robot_sms(
        &self,
        config: &WecomRobotConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("企业微信群机器人 skipped".to_string());
        }
        let text = render_sms_template(&config.common.sms_template, message, false);
        self.send_wecom_robot_text(config, text).await
    }

    async fn send_wecom_robot_call(
        &self,
        config: &WecomRobotConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("企业微信群机器人 skipped".to_string());
        }
        let text = render_call_template(&config.common.call_template, call, false);
        self.send_wecom_robot_text(config, text).await
    }

    async fn send_wecom_robot_ddns(
        &self,
        config: &WecomRobotConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("企业微信群机器人 skipped".to_string());
        }
        let text = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_wecom_robot_text(config, text).await
    }

    async fn send_wecom_robot_text(
        &self,
        config: &WecomRobotConfig,
        text: String,
    ) -> Result<String, String> {
        let url = robot_webhook_url(
            &config.webhook_url,
            &config.key,
            "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=",
        )?;
        let payload = json!({
            "msgtype": "text",
            "text": { "content": text },
        });

        self.post_json("企业微信群机器人", &url, payload).await
    }

    async fn send_dingtalk_robot_sms(
        &self,
        config: &DingtalkRobotConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("钉钉群自定义机器人 skipped".to_string());
        }
        let text = render_sms_template(&config.common.sms_template, message, false);
        self.send_dingtalk_robot_text(config, text).await
    }

    async fn send_dingtalk_robot_call(
        &self,
        config: &DingtalkRobotConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("钉钉群自定义机器人 skipped".to_string());
        }
        let text = render_call_template(&config.common.call_template, call, false);
        self.send_dingtalk_robot_text(config, text).await
    }

    async fn send_dingtalk_robot_ddns(
        &self,
        config: &DingtalkRobotConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("钉钉群自定义机器人 skipped".to_string());
        }
        let text = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_dingtalk_robot_text(config, text).await
    }

    async fn send_dingtalk_robot_text(
        &self,
        config: &DingtalkRobotConfig,
        text: String,
    ) -> Result<String, String> {
        let mut url = robot_webhook_url(
            &config.webhook_url,
            &config.access_token,
            "https://oapi.dingtalk.com/robot/send?access_token=",
        )?;
        if !config.secret.trim().is_empty() {
            let timestamp = current_timestamp_millis();
            let to_sign = format!("{}\n{}", timestamp, config.secret.trim());
            let sign = hmac_sha256_base64(config.secret.trim().as_bytes(), to_sign.as_bytes());
            let separator = if url.contains('?') { '&' } else { '?' };
            url.push_str(&format!(
                "{}timestamp={}&sign={}",
                separator,
                timestamp,
                encode_query_value(&sign)
            ));
        }

        let at_mobiles = split_csv(&config.at_mobiles);
        let payload = json!({
            "msgtype": "text",
            "text": { "content": text },
            "at": {
                "atMobiles": at_mobiles,
                "isAtAll": config.at_all,
            },
        });

        self.post_json("钉钉群自定义机器人", &url, payload).await
    }

    async fn send_dingtalk_app_sms(
        &self,
        config: &DingtalkAppConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("钉钉企业内机器人 skipped".to_string());
        }
        let text = render_sms_template(&config.common.sms_template, message, false);
        self.send_dingtalk_app_text(config, text).await
    }

    async fn send_dingtalk_app_call(
        &self,
        config: &DingtalkAppConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("钉钉企业内机器人 skipped".to_string());
        }
        let text = render_call_template(&config.common.call_template, call, false);
        self.send_dingtalk_app_text(config, text).await
    }

    async fn send_dingtalk_app_ddns(
        &self,
        config: &DingtalkAppConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("钉钉企业内部机器人 skipped".to_string());
        }
        let text = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_dingtalk_app_text(config, text).await
    }

    async fn send_dingtalk_app_text(
        &self,
        config: &DingtalkAppConfig,
        text: String,
    ) -> Result<String, String> {
        if config.app_key.trim().is_empty()
            || config.app_secret.trim().is_empty()
            || config.open_conversation_id.trim().is_empty()
        {
            return Err("钉钉 AppKey、AppSecret 或 OpenConversationId 未配置".to_string());
        }
        let token = self
            .fetch_dingtalk_access_token(config.app_key.trim(), config.app_secret.trim())
            .await?;
        let robot_code = if config.robot_code.trim().is_empty() {
            config.app_key.trim()
        } else {
            config.robot_code.trim()
        };
        let msg_key = if config.msg_key.trim().is_empty() {
            "sampleText"
        } else {
            config.msg_key.trim()
        };
        let msg_param = json!({ "content": text }).to_string();
        let payload = json!({
            "robotCode": robot_code,
            "openConversationId": config.open_conversation_id.trim(),
            "msgKey": msg_key,
            "msgParam": msg_param,
        });

        let response = self
            .client
            .post("https://api.dingtalk.com/v1.0/robot/groupMessages/send")
            .header("x-acs-dingtalk-access-token", token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send DingTalk app robot message: {}", e))?;
        response_result(
            "钉钉企业内机器人",
            response.status(),
            response.text().await.unwrap_or_default(),
        )
    }

    async fn fetch_dingtalk_access_token(
        &self,
        app_key: &str,
        app_secret: &str,
    ) -> Result<String, String> {
        #[derive(Debug, Deserialize)]
        struct DingtalkTokenResponse {
            #[serde(default, rename = "accessToken")]
            access_token: String,
            #[serde(default)]
            code: String,
            #[serde(default)]
            message: String,
        }

        let payload = json!({
            "appKey": app_key,
            "appSecret": app_secret,
        });
        let response = self
            .client
            .post("https://api.dingtalk.com/v1.0/oauth2/accessToken")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to get DingTalk access token: {}", e))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!(
                "DingTalk token request failed ({}): {}",
                status, body
            ));
        }
        let parsed: DingtalkTokenResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse DingTalk token response: {}", e))?;
        if !parsed.access_token.is_empty() {
            return Ok(parsed.access_token);
        }
        Err(format!(
            "DingTalk token response did not include accessToken: {} {}",
            parsed.code, parsed.message
        ))
    }

    async fn send_feishu_robot_sms(
        &self,
        config: &FeishuRobotConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("飞书机器人 skipped".to_string());
        }
        let text = render_sms_template(&config.common.sms_template, message, false);
        self.send_feishu_robot_text(config, text).await
    }

    async fn send_feishu_robot_call(
        &self,
        config: &FeishuRobotConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("飞书机器人 skipped".to_string());
        }
        let text = render_call_template(&config.common.call_template, call, false);
        self.send_feishu_robot_text(config, text).await
    }

    async fn send_feishu_robot_ddns(
        &self,
        config: &FeishuRobotConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("飞书机器人 skipped".to_string());
        }
        let text = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_feishu_robot_text(config, text).await
    }

    async fn send_feishu_robot_text(
        &self,
        config: &FeishuRobotConfig,
        text: String,
    ) -> Result<String, String> {
        let url = robot_webhook_url(
            &config.webhook_url,
            &config.token,
            "https://open.feishu.cn/open-apis/bot/v2/hook/",
        )?;
        let mut payload = Map::new();
        payload.insert("msg_type".to_string(), json!("text"));
        payload.insert("content".to_string(), json!({ "text": text }));
        if !config.secret.trim().is_empty() {
            let timestamp = current_timestamp_secs().to_string();
            let sign_key = format!("{}\n{}", timestamp, config.secret.trim());
            let sign = hmac_sha256_base64(sign_key.as_bytes(), b"");
            payload.insert("timestamp".to_string(), json!(timestamp));
            payload.insert("sign".to_string(), json!(sign));
        }

        self.post_json("飞书机器人", &url, Value::Object(payload))
            .await
    }

    async fn send_telegram_sms(
        &self,
        config: &TelegramConfig,
        message: &SmsMessage,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_sms(&config.common, force) {
            return Ok("Telegram skipped".to_string());
        }
        let text = render_sms_template(&config.common.sms_template, message, false);
        self.send_telegram_text(config, text).await
    }

    async fn send_telegram_call(
        &self,
        config: &TelegramConfig,
        call: &CallRecord,
        force: bool,
    ) -> Result<String, String> {
        if !should_send_call(&config.common, force) {
            return Ok("Telegram skipped".to_string());
        }
        let text = render_call_template(&config.common.call_template, call, false);
        self.send_telegram_text(config, text).await
    }

    async fn send_telegram_ddns(
        &self,
        config: &TelegramConfig,
        event: &DdnsEvent,
    ) -> Result<String, String> {
        if !should_send_ddns(&config.common) {
            return Ok("Telegram skipped".to_string());
        }
        let text = render_ddns_template(&config.common.ddns_template, event, false);
        self.send_telegram_text(config, text).await
    }

    async fn send_telegram_text(
        &self,
        config: &TelegramConfig,
        text: String,
    ) -> Result<String, String> {
        if config.bot_token.trim().is_empty() || config.chat_id.trim().is_empty() {
            return Err("Telegram Bot Token 或 Chat ID 未配置".to_string());
        }
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            config.bot_token.trim()
        );
        let mut payload = Map::new();
        payload.insert("chat_id".to_string(), json!(config.chat_id.trim()));
        payload.insert("text".to_string(), json!(text));
        payload.insert(
            "disable_web_page_preview".to_string(),
            json!(config.disable_web_page_preview),
        );
        insert_non_empty(&mut payload, "parse_mode", &config.parse_mode);

        self.post_json("Telegram", &url, Value::Object(payload))
            .await
    }

    async fn post_json(&self, label: &str, url: &str, payload: Value) -> Result<String, String> {
        let response = self
            .client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send {} message: {}", label, e))?;
        response_result(
            label,
            response.status(),
            response.text().await.unwrap_or_default(),
        )
    }
}

impl NotificationChannel {
    fn label(self) -> &'static str {
        match self {
            NotificationChannel::Webhook => "Webhook",
            NotificationChannel::Bark => "Bark",
            NotificationChannel::PushPlus => "PushPlus",
            NotificationChannel::WecomApp => "企业微信应用消息",
            NotificationChannel::WecomRobot => "企业微信群机器人",
            NotificationChannel::DingtalkRobot => "钉钉群自定义机器人",
            NotificationChannel::DingtalkApp => "钉钉企业内机器人",
            NotificationChannel::FeishuRobot => "飞书机器人",
            NotificationChannel::Telegram => "Telegram机器人",
        }
    }
}

fn all_channels() -> [NotificationChannel; 9] {
    [
        NotificationChannel::Webhook,
        NotificationChannel::Bark,
        NotificationChannel::PushPlus,
        NotificationChannel::WecomApp,
        NotificationChannel::WecomRobot,
        NotificationChannel::DingtalkRobot,
        NotificationChannel::DingtalkApp,
        NotificationChannel::FeishuRobot,
        NotificationChannel::Telegram,
    ]
}

fn should_send_sms(config: &MessageChannelConfig, force: bool) -> bool {
    force || (config.enabled && config.forward_sms)
}

fn should_send_call(config: &MessageChannelConfig, force: bool) -> bool {
    force || (config.enabled && config.forward_calls)
}

fn should_send_ddns(config: &MessageChannelConfig) -> bool {
    config.enabled && config.forward_ddns
}

const DEFAULT_DDNS_TEXT_TEMPLATE: &str = "SimAdmin DDNS 通知\n域名: {{domains}}\nIP类型: {{ip_type}}\n新IP: {{new_ip}}\n旧IP: {{old_ip}}\n服务商: {{provider}}\n记录类型: {{record_type}}\n状态: {{status}}\n消息: {{message}}\n更新时间: {{timestamp}}";
const DEFAULT_DDNS_JSON_TEMPLATE: &str = r#"{
  "msg_type": "text",
  "content": {
    "text": "SimAdmin DDNS 通知\n域名: {{domains}}\nIP类型: {{ip_type}}\n新IP: {{new_ip}}\n旧IP: {{old_ip}}\n服务商: {{provider}}\n记录类型: {{record_type}}\n状态: {{status}}\n消息: {{message}}\n更新时间: {{timestamp}}"
  }
}"#;

fn render_ddns_template(template: &str, event: &DdnsEvent, escape_json: bool) -> String {
    let domains = if event.domains.is_empty() {
        "-".to_string()
    } else {
        event.domains.join(", ")
    };
    let ip_type = match event.record_type.as_str() {
        "A" => "IPv4",
        "AAAA" => "IPv6",
        other => other,
    };
    let old_ip = event.old_ip.as_deref().unwrap_or("-").to_string();
    let new_ip = event.new_ip.as_deref().unwrap_or("-").to_string();
    let template = if template.trim().is_empty() && escape_json {
        DEFAULT_DDNS_JSON_TEMPLATE
    } else if template.trim().is_empty() {
        DEFAULT_DDNS_TEXT_TEMPLATE
    } else {
        template
    };

    let maybe_escape = |value: &str| {
        if escape_json {
            escape_json_string(value)
        } else {
            value.to_string()
        }
    };
    let domains = maybe_escape(&domains);
    let ip_type = maybe_escape(ip_type);
    let old_ip = maybe_escape(&old_ip);
    let new_ip = maybe_escape(&new_ip);
    let provider = maybe_escape(&event.provider);
    let record_type = maybe_escape(&event.record_type);
    let status = maybe_escape(&event.status);
    let message = maybe_escape(&event.message);
    let timestamp_value = format_notification_time(&event.timestamp);
    let timestamp = maybe_escape(&timestamp_value);

    template
        .replace("{{domains}}", &domains)
        .replace("{{domain}}", &domains)
        .replace("{{ip_type}}", &ip_type)
        .replace("{{new_ip}}", &new_ip)
        .replace("{{old_ip}}", &old_ip)
        .replace("{{provider}}", &provider)
        .replace("{{record_type}}", &record_type)
        .replace("{{status}}", &status)
        .replace("{{message}}", &message)
        .replace("{{timestamp}}", &timestamp)
        .replace("{{time}}", &timestamp)
        .replace("{{域名}}", &domains)
        .replace("{{IP类型}}", &ip_type)
        .replace("{{新IP}}", &new_ip)
        .replace("{{旧IP}}", &old_ip)
        .replace("{{服务商}}", &provider)
        .replace("{{记录类型}}", &record_type)
        .replace("{{状态}}", &status)
        .replace("{{消息}}", &message)
        .replace("{{更新时间}}", &timestamp)
}

fn robot_webhook_url(webhook_url: &str, key: &str, prefix: &str) -> Result<String, String> {
    let webhook_url = webhook_url.trim();
    if !webhook_url.is_empty() {
        return Ok(webhook_url.to_string());
    }
    let key = key.trim();
    if key.is_empty() {
        return Err("Webhook URL 或 Key/Token 未配置".to_string());
    }
    Ok(format!("{}{}", prefix, encode_path_segment(key)))
}

fn split_csv(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn insert_non_empty(payload: &mut Map<String, Value>, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        payload.insert(key.to_string(), json!(value));
    }
}

fn encode_query_value(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn encode_path_segment(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn hmac_sha256_base64(key: &[u8], data: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let tag = hmac::sign(&key, data);
    general_purpose::STANDARD.encode(tag.as_ref())
}

fn is_wecom_access_token_error(body: &str) -> bool {
    json_errcode(body)
        .map(|(errcode, _)| matches!(errcode, 40014 | 42001))
        .unwrap_or(false)
}

fn json_errcode(body: &str) -> Option<(i64, String)> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    let errcode = value.get("errcode").and_then(Value::as_i64)?;
    let message = value
        .get("errmsg")
        .or_else(|| value.get("err_msg"))
        .and_then(Value::as_str)
        .unwrap_or(body)
        .to_string();
    Some((errcode, message))
}

fn response_result(label: &str, status: StatusCode, body: String) -> Result<String, String> {
    if !status.is_success() {
        return Err(format!("{} returned HTTP {}: {}", label, status, body));
    }

    if let Ok(value) = serde_json::from_str::<Value>(&body) {
        if let Some(ok) = value.get("ok").and_then(Value::as_bool) {
            if !ok {
                return Err(format!("{} returned error: {}", label, body));
            }
        }
        if let Some(errcode) = value.get("errcode").and_then(Value::as_i64) {
            if errcode != 0 {
                let message = value
                    .get("errmsg")
                    .or_else(|| value.get("err_msg"))
                    .and_then(Value::as_str)
                    .unwrap_or(&body);
                return Err(format!(
                    "{} returned errcode {}: {}",
                    label, errcode, message
                ));
            }
        }
        if let Some(code) = value.get("code").and_then(Value::as_i64) {
            if code != 0 && code != 200 {
                let message = value
                    .get("msg")
                    .or_else(|| value.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or(&body);
                return Err(format!("{} returned code {}: {}", label, code, message));
            }
        }
        if let Some(status_code) = value.get("StatusCode").and_then(Value::as_i64) {
            if status_code != 0 {
                let message = value
                    .get("StatusMessage")
                    .and_then(Value::as_str)
                    .unwrap_or(&body);
                return Err(format!(
                    "{} returned StatusCode {}: {}",
                    label, status_code, message
                ));
            }
        }
    }

    Ok(format!("{} test successful (status: {})", label, status))
}

fn render_sms_template(template: &str, message: &SmsMessage, escape_json: bool) -> String {
    let content = if escape_json {
        escape_json_string(&message.content)
    } else {
        message.content.clone()
    };
    let timestamp = render_time_value(&message.timestamp, escape_json);

    template
        .replace("{{id}}", &message.id.to_string())
        .replace("{{phone_number}}", &message.phone_number)
        .replace("{{content}}", &content)
        .replace("{{direction}}", &message.direction)
        .replace("{{timestamp}}", &timestamp)
        .replace("{{status}}", &message.status)
        .replace("{{sender}}", &message.phone_number)
        .replace("{{message}}", &content)
        .replace("{{time}}", &timestamp)
}

fn render_call_template(template: &str, call: &CallRecord, escape_json: bool) -> String {
    let start_time = render_time_value(&call.start_time, escape_json);
    let end_time = call
        .end_time
        .as_deref()
        .map(|value| render_time_value(value, escape_json))
        .unwrap_or_default();
    let answered_str = if call.answered { "是" } else { "否" };
    let answered_value = if escape_json {
        escape_json_string(answered_str)
    } else {
        answered_str.to_string()
    };
    let direction_cn = if call.direction == "incoming" {
        "来电"
    } else {
        "去电"
    };

    template
        .replace("{{id}}", &call.id.to_string())
        .replace("{{phone_number}}", &call.phone_number)
        .replace("{{direction}}", &call.direction)
        .replace("{{direction_cn}}", direction_cn)
        .replace("{{duration}}", &call.duration.to_string())
        .replace("{{start_time}}", &start_time)
        .replace("{{end_time}}", &end_time)
        .replace("{{answered}}", &answered_value)
        .replace("{{answered_bool}}", &call.answered.to_string())
        .replace("{{caller}}", &call.phone_number)
        .replace("{{time}}", &start_time)
}

fn render_time_value(value: &str, escape_json: bool) -> String {
    let formatted = format_notification_time(value);
    if escape_json {
        escape_json_string(&formatted)
    } else {
        formatted
    }
}

fn beijing_now_string() -> String {
    Utc::now()
        .with_timezone(&beijing_offset())
        .format(NOTIFICATION_TIME_FORMAT)
        .to_string()
}

fn format_notification_time(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }

    if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
        return datetime
            .with_timezone(&beijing_offset())
            .format(NOTIFICATION_TIME_FORMAT)
            .to_string();
    }

    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, format) {
            return datetime.format(NOTIFICATION_TIME_FORMAT).to_string();
        }
    }

    value.to_string()
}

fn beijing_offset() -> FixedOffset {
    FixedOffset::east_opt(BEIJING_UTC_OFFSET_SECONDS).expect("valid Beijing UTC offset")
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn compute_legacy_signature(secret: &str, data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}{}", secret, data).hash(&mut hasher);
    let hash = hasher.finish();

    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_rfc3339_time_as_beijing_time() {
        assert_eq!(
            format_notification_time("2026-05-14T16:30:45Z"),
            "2026-05-15 00:30:45"
        );
        assert_eq!(
            format_notification_time("2026-05-15T08:30:45+08:00"),
            "2026-05-15 08:30:45"
        );
    }

    #[test]
    fn renders_sms_time_variables_as_beijing_time() {
        let message = SmsMessage {
            id: 7,
            direction: "incoming".to_string(),
            phone_number: "+8613800138000".to_string(),
            content: "hello".to_string(),
            timestamp: "2026-05-14T16:30:45Z".to_string(),
            status: "received".to_string(),
            pdu: None,
        };

        assert_eq!(
            render_sms_template("{{timestamp}}|{{time}}", &message, false),
            "2026-05-15 00:30:45|2026-05-15 00:30:45"
        );
    }

    #[test]
    fn renders_call_time_variables_as_beijing_time() {
        let call = CallRecord {
            id: 9,
            direction: "incoming".to_string(),
            phone_number: "+8613800138000".to_string(),
            duration: 12,
            start_time: "2026-05-14T16:30:45Z".to_string(),
            end_time: Some("2026-05-14T16:31:45Z".to_string()),
            answered: true,
        };

        assert_eq!(
            render_call_template("{{start_time}}|{{end_time}}|{{time}}", &call, false),
            "2026-05-15 00:30:45|2026-05-15 00:31:45|2026-05-15 00:30:45"
        );
    }

    #[test]
    fn renders_ddns_time_variables_as_beijing_time() {
        let event = DdnsEvent {
            timestamp: "2026-05-14T16:30:45Z".to_string(),
            ..DdnsEvent::default()
        };

        assert_eq!(
            render_ddns_template("{{timestamp}}|{{time}}|{{更新时间}}", &event, false),
            "2026-05-15 00:30:45|2026-05-15 00:30:45|2026-05-15 00:30:45"
        );
    }

    #[test]
    fn detects_wecom_access_token_errors() {
        assert!(is_wecom_access_token_error(
            r#"{"errcode":40014,"errmsg":"invalidaccess_token"}"#
        ));
        assert!(is_wecom_access_token_error(
            r#"{"errcode":42001,"errmsg":"access_token expired"}"#
        ));
        assert!(!is_wecom_access_token_error(
            r#"{"errcode":0,"errmsg":"ok"}"#
        ));
    }
}
