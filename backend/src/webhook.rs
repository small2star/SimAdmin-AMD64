//! Notification forwarding module.
//!
//! Keeps the historical `WebhookSender` type name while supporting multiple
//! notification channels configured from the notification center.

use crate::config::{
    BarkConfig, ConfigManager, DingtalkAppConfig, DingtalkRobotConfig, FeishuRobotConfig,
    MessageChannelConfig, NotificationChannel, NotificationConfig, TelegramConfig, WebhookConfig,
    WecomAppConfig, WecomRobotConfig,
};
use crate::db::{CallRecord, SmsMessage};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{Client, StatusCode};
use ring::hmac;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Webhook sender.
pub struct WebhookSender {
    client: Client,
    config_manager: Arc<ConfigManager>,
}

impl WebhookSender {
    /// Create a new sender.
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            config_manager,
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

    /// Test a specific notification channel with a simulated SMS.
    pub async fn test_channel(&self, channel: NotificationChannel) -> Result<String, String> {
        let config = self.get_config();
        let test_message = SmsMessage {
            id: 0,
            direction: "incoming".to_string(),
            phone_number: "+8613800138000".to_string(),
            content: "这是一条测试短信 (Notification Test)".to_string(),
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
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

        let token = self
            .fetch_wecom_access_token(config.corp_id.trim(), config.secret.trim())
            .await?;
        let url = format!(
            "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={}",
            encode_query_value(&token)
        );
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

        self.post_json("企业微信应用消息", &url, payload).await
    }

    async fn fetch_wecom_access_token(
        &self,
        corp_id: &str,
        secret: &str,
    ) -> Result<String, String> {
        #[derive(Debug, Deserialize)]
        struct WecomTokenResponse {
            errcode: i64,
            #[serde(default)]
            errmsg: String,
            #[serde(default)]
            access_token: String,
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
        let parsed: WecomTokenResponse = serde_json::from_str(&body)
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
        Ok(parsed.access_token)
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
            NotificationChannel::WecomApp => "企业微信应用消息",
            NotificationChannel::WecomRobot => "企业微信群机器人",
            NotificationChannel::DingtalkRobot => "钉钉群自定义机器人",
            NotificationChannel::DingtalkApp => "钉钉企业内机器人",
            NotificationChannel::FeishuRobot => "飞书机器人",
            NotificationChannel::Telegram => "Telegram机器人",
        }
    }
}

fn all_channels() -> [NotificationChannel; 8] {
    [
        NotificationChannel::Webhook,
        NotificationChannel::Bark,
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

    template
        .replace("{{id}}", &message.id.to_string())
        .replace("{{phone_number}}", &message.phone_number)
        .replace("{{content}}", &content)
        .replace("{{direction}}", &message.direction)
        .replace("{{timestamp}}", &message.timestamp)
        .replace("{{status}}", &message.status)
        .replace("{{sender}}", &message.phone_number)
        .replace("{{message}}", &content)
        .replace("{{time}}", &message.timestamp)
}

fn render_call_template(template: &str, call: &CallRecord, escape_json: bool) -> String {
    let end_time = call.end_time.clone().unwrap_or_default();
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
        .replace("{{start_time}}", &call.start_time)
        .replace("{{end_time}}", &end_time)
        .replace("{{answered}}", &answered_value)
        .replace("{{answered_bool}}", &call.answered.to_string())
        .replace("{{caller}}", &call.phone_number)
        .replace("{{time}}", &call.start_time)
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
