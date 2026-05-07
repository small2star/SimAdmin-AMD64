import { useCallback, useEffect, useMemo, useRef, useState, type ChangeEvent } from 'react'
import {
  Alert,
  Avatar,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Divider,
  FormControlLabel,
  IconButton,
  List,
  ListItemButton,
  ListItemText,
  MenuItem,
  Paper,
  Snackbar,
  Switch,
  TextField,
  Tooltip,
  Typography,
  useMediaQuery,
} from '@mui/material'
import { alpha, type Theme } from '@mui/material/styles'
import {
  Add,
  ArrowBack,
  Business,
  Chat,
  DeleteOutline,
  Forum,
  Groups,
  NotificationsActive,
  PhoneIphone,
  PlayArrow,
  Save,
  Send,
  Webhook,
  Work,
} from '@mui/icons-material'
import { api, type NotificationChannelKey, type NotificationConfig } from '../api/current'
import {
  DEFAULT_CALL_TEMPLATE,
  DEFAULT_PLAIN_CALL_TEMPLATE,
  DEFAULT_PLAIN_SMS_TEMPLATE,
  DEFAULT_SMS_TEMPLATE,
} from '../api/current'
import ErrorSnackbar from '../components/ErrorSnackbar'

const CHANNEL_KEYS: NotificationChannelKey[] = [
  'webhook',
  'bark',
  'wecom_app',
  'wecom_robot',
  'dingtalk_robot',
  'dingtalk_app',
  'feishu_robot',
  'telegram',
]

const CHANNEL_DEFS = [
  { key: 'webhook', label: 'Webhook', icon: Webhook },
  { key: 'bark', label: 'Bark', icon: PhoneIphone },
  { key: 'wecom_app', label: '企业微信应用消息', icon: Business },
  { key: 'wecom_robot', label: '企业微信群机器人', icon: Groups },
  { key: 'dingtalk_robot', label: '钉钉群自定义机器人', icon: Forum },
  { key: 'dingtalk_app', label: '钉钉企业内机器人', icon: Work },
  { key: 'feishu_robot', label: '飞书机器人', icon: Chat },
  { key: 'telegram', label: 'Telegram机器人', icon: Send },
] as const

const LEGACY_MOJIBAKE_SMS_TEMPLATE = '{\n'
  + '  "msg_type": "text",\n'
  + '  "content": {\n'
  + '    "text": "\u9983\u646b \u942d\ue15d\u4fca\u95ab\u6c31\u7161\\\\n\u9359\u6226\u20ac\u4f79\u67df: {{phone_number}}\\\\n\u9350\u546d\ue190: {{content}}\\\\n\u93c3\u5815\u68ff: {{timestamp}}"\n'
  + '  }\n'
  + '}'

const LEGACY_MOJIBAKE_CALL_TEMPLATE = '{\n'
  + '  "msg_type": "text",\n'
  + '  "content": {\n'
  + '    "text": "\u9983\u6453 \u93c9\u30e7\u6578\u95ab\u6c31\u7161\\\\n\u9359\u98ce\u721c: {{phone_number}}\\\\n\u7eeb\u8bf2\u7037: {{direction}}\\\\n\u93c3\u5815\u68ff: {{start_time}}\\\\n\u93c3\u5815\u66b1: {{duration}}\u7ec9\u6283\\\\n\u5bb8\u53c9\u5e34\u935a? {{answered}}"\n'
  + '  }\n'
  + '}'

const SMS_TEMPLATE_VARIABLES = [
  { label: '发送方号码', displayToken: '{{发送方号码}}', backendToken: '{{phone_number}}' },
  { label: '短信内容', displayToken: '{{短信内容}}', backendToken: '{{content}}' },
  { label: '时间', displayToken: '{{时间}}', backendToken: '{{timestamp}}' },
  { label: '短信方向', displayToken: '{{短信方向}}', backendToken: '{{direction}}' },
  { label: '短信状态', displayToken: '{{短信状态}}', backendToken: '{{status}}' },
] as const

const CALL_TEMPLATE_VARIABLES = [
  { label: '来电号码', displayToken: '{{来电号码}}', backendToken: '{{phone_number}}' },
  { label: '通话时长', displayToken: '{{通话时长}}', backendToken: '{{duration}}' },
  { label: '开始时间', displayToken: '{{开始时间}}', backendToken: '{{start_time}}' },
  { label: '结束时间', displayToken: '{{结束时间}}', backendToken: '{{end_time}}' },
  { label: '是否接听', displayToken: '{{是否接听}}', backendToken: '{{answered}}' },
  { label: '通话方向', displayToken: '{{通话方向}}', backendToken: '{{direction}}' },
] as const

function replaceAll(input: string, search: string, replacement: string) {
  return input.split(search).join(replacement)
}

function toDisplayTemplate(template: string, variables: readonly { displayToken: string; backendToken: string }[]) {
  return variables.reduce(
    (result, variable) => replaceAll(result, variable.backendToken, variable.displayToken),
    template,
  )
}

function toBackendTemplate(template: string, variables: readonly { displayToken: string; backendToken: string }[]) {
  return variables.reduce(
    (result, variable) => replaceAll(result, variable.displayToken, variable.backendToken),
    template,
  )
}

const DEFAULT_SMS_DISPLAY_TEMPLATE = toDisplayTemplate(DEFAULT_SMS_TEMPLATE, SMS_TEMPLATE_VARIABLES)
const DEFAULT_CALL_DISPLAY_TEMPLATE = toDisplayTemplate(DEFAULT_CALL_TEMPLATE, CALL_TEMPLATE_VARIABLES)
const DEFAULT_PLAIN_SMS_DISPLAY_TEMPLATE = toDisplayTemplate(DEFAULT_PLAIN_SMS_TEMPLATE, SMS_TEMPLATE_VARIABLES)
const DEFAULT_PLAIN_CALL_DISPLAY_TEMPLATE = toDisplayTemplate(DEFAULT_PLAIN_CALL_TEMPLATE, CALL_TEMPLATE_VARIABLES)

function baseMessageConfig() {
  return {
    enabled: false,
    forward_sms: true,
    forward_calls: true,
    sms_template: DEFAULT_PLAIN_SMS_DISPLAY_TEMPLATE,
    call_template: DEFAULT_PLAIN_CALL_DISPLAY_TEMPLATE,
  }
}

function createDefaultNotificationConfig(): NotificationConfig {
  return {
    webhook: {
      enabled: false,
      url: '',
      forward_sms: true,
      forward_calls: true,
      headers: {},
      secret: '',
      sms_template: DEFAULT_SMS_DISPLAY_TEMPLATE,
      call_template: DEFAULT_CALL_DISPLAY_TEMPLATE,
    },
    bark: {
      ...baseMessageConfig(),
      server_url: 'https://api.day.app',
      device_key: '',
      title_template: 'SimAdmin 短信通知',
      group: '',
      sound: '',
      level: '',
      icon: '',
      click_url: '',
      copy: '',
      auto_copy: false,
      save_history: true,
    },
    wecom_app: {
      ...baseMessageConfig(),
      corp_id: '',
      agent_id: '',
      secret: '',
      to_user: '@all',
      to_party: '',
      to_tag: '',
      safe: false,
    },
    wecom_robot: {
      ...baseMessageConfig(),
      webhook_url: '',
      key: '',
    },
    dingtalk_robot: {
      ...baseMessageConfig(),
      webhook_url: '',
      access_token: '',
      secret: '',
      at_mobiles: '',
      at_all: false,
    },
    dingtalk_app: {
      ...baseMessageConfig(),
      app_key: '',
      app_secret: '',
      robot_code: '',
      open_conversation_id: '',
      msg_key: 'sampleText',
    },
    feishu_robot: {
      ...baseMessageConfig(),
      webhook_url: '',
      token: '',
      secret: '',
    },
    telegram: {
      ...baseMessageConfig(),
      bot_token: '',
      chat_id: '',
      parse_mode: '',
      disable_web_page_preview: true,
    },
  }
}

function withTemplateDisplay<T extends { sms_template: string; call_template: string }>(channel: T): T {
  return {
    ...channel,
    sms_template: toDisplayTemplate(channel.sms_template, SMS_TEMPLATE_VARIABLES),
    call_template: toDisplayTemplate(channel.call_template, CALL_TEMPLATE_VARIABLES),
  }
}

function withTemplateBackend<T extends { sms_template: string; call_template: string }>(channel: T): T {
  return {
    ...channel,
    sms_template: toBackendTemplate(channel.sms_template, SMS_TEMPLATE_VARIABLES),
    call_template: toBackendTemplate(channel.call_template, CALL_TEMPLATE_VARIABLES),
  }
}

function mergeNotificationConfig(config?: NotificationConfig): NotificationConfig {
  const defaults = createDefaultNotificationConfig()
  if (!config) return defaults

  return {
    webhook: { ...defaults.webhook, ...config.webhook, headers: config.webhook?.headers ?? {} },
    bark: { ...defaults.bark, ...config.bark },
    wecom_app: { ...defaults.wecom_app, ...config.wecom_app },
    wecom_robot: { ...defaults.wecom_robot, ...config.wecom_robot },
    dingtalk_robot: { ...defaults.dingtalk_robot, ...config.dingtalk_robot },
    dingtalk_app: { ...defaults.dingtalk_app, ...config.dingtalk_app },
    feishu_robot: { ...defaults.feishu_robot, ...config.feishu_robot },
    telegram: { ...defaults.telegram, ...config.telegram },
  }
}

function normalizeNotificationConfig(config?: NotificationConfig): NotificationConfig {
  const merged = mergeNotificationConfig(config)
  const webhookSmsTemplate = merged.webhook.sms_template === LEGACY_MOJIBAKE_SMS_TEMPLATE
    ? DEFAULT_SMS_TEMPLATE
    : merged.webhook.sms_template
  const webhookCallTemplate = merged.webhook.call_template === LEGACY_MOJIBAKE_CALL_TEMPLATE
    ? DEFAULT_CALL_TEMPLATE
    : merged.webhook.call_template

  return {
    webhook: withTemplateDisplay({
      ...merged.webhook,
      sms_template: webhookSmsTemplate,
      call_template: webhookCallTemplate,
    }),
    bark: withTemplateDisplay(merged.bark),
    wecom_app: withTemplateDisplay(merged.wecom_app),
    wecom_robot: withTemplateDisplay(merged.wecom_robot),
    dingtalk_robot: withTemplateDisplay(merged.dingtalk_robot),
    dingtalk_app: withTemplateDisplay(merged.dingtalk_app),
    feishu_robot: withTemplateDisplay(merged.feishu_robot),
    telegram: withTemplateDisplay(merged.telegram),
  }
}

function getBackendNotificationConfig(config: NotificationConfig): NotificationConfig {
  return {
    webhook: withTemplateBackend(config.webhook),
    bark: withTemplateBackend(config.bark),
    wecom_app: withTemplateBackend(config.wecom_app),
    wecom_robot: withTemplateBackend(config.wecom_robot),
    dingtalk_robot: withTemplateBackend(config.dingtalk_robot),
    dingtalk_app: withTemplateBackend(config.dingtalk_app),
    feishu_robot: withTemplateBackend(config.feishu_robot),
    telegram: withTemplateBackend(config.telegram),
  }
}

function isChannelConfigured(channel: NotificationChannelKey, config: NotificationConfig) {
  switch (channel) {
    case 'webhook':
      return Boolean(config.webhook.url.trim())
    case 'bark':
      return Boolean(config.bark.device_key.trim())
    case 'wecom_app':
      return Boolean(config.wecom_app.corp_id.trim() && config.wecom_app.agent_id.trim() && config.wecom_app.secret.trim())
    case 'wecom_robot':
      return Boolean(config.wecom_robot.webhook_url.trim() || config.wecom_robot.key.trim())
    case 'dingtalk_robot':
      return Boolean(config.dingtalk_robot.webhook_url.trim() || config.dingtalk_robot.access_token.trim())
    case 'dingtalk_app':
      return Boolean(config.dingtalk_app.app_key.trim() && config.dingtalk_app.app_secret.trim() && config.dingtalk_app.open_conversation_id.trim())
    case 'feishu_robot':
      return Boolean(config.feishu_robot.webhook_url.trim() || config.feishu_robot.token.trim())
    case 'telegram':
      return Boolean(config.telegram.bot_token.trim() && config.telegram.chat_id.trim())
    default:
      return false
  }
}

export default function NotificationCenterPage() {
  const isMobile = useMediaQuery<Theme>((theme: Theme) => theme.breakpoints.down('md'))
  const [config, setConfig] = useState<NotificationConfig>(() => createDefaultNotificationConfig())
  const [selectedChannel, setSelectedChannel] = useState<NotificationChannelKey>('webhook')
  const [showMobileList, setShowMobileList] = useState(true)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [testing, setTesting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [newHeaderKey, setNewHeaderKey] = useState('')
  const [newHeaderValue, setNewHeaderValue] = useState('')
  const smsTemplateInputRef = useRef<HTMLTextAreaElement | null>(null)

  const loadConfig = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const response = await api.getNotificationConfig()
      if (response.data) {
        setConfig(normalizeNotificationConfig(response.data))
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadConfig()
  }, [loadConfig])

  const selectedDef = CHANNEL_DEFS.find((item) => item.key === selectedChannel) ?? CHANNEL_DEFS[0]
  const SelectedIcon = selectedDef.icon
  const selectedConfig = config[selectedChannel]
  const enabledCount = useMemo(
    () => CHANNEL_KEYS.filter((channel) => config[channel].enabled).length,
    [config],
  )

  const patchChannel = (channel: NotificationChannelKey, patch: Record<string, unknown>) => {
    setConfig((prev) => ({
      ...prev,
      [channel]: {
        ...prev[channel],
        ...patch,
      },
    } as NotificationConfig))
  }

  const setChannelEnabled = (channel: NotificationChannelKey, enabled: boolean) => {
    patchChannel(channel, { enabled })
  }

  const handleSelectChannel = (channel: NotificationChannelKey) => {
    setSelectedChannel(channel)
    if (isMobile) {
      setShowMobileList(false)
    }
  }

  const handleAddHeader = () => {
    if (!newHeaderKey.trim() || !newHeaderValue.trim()) return
    patchChannel('webhook', {
      headers: {
        ...config.webhook.headers,
        [newHeaderKey.trim()]: newHeaderValue.trim(),
      },
    })
    setNewHeaderKey('')
    setNewHeaderValue('')
  }

  const handleRemoveHeader = (key: string) => {
    const headers = { ...config.webhook.headers }
    delete headers[key]
    patchChannel('webhook', { headers })
  }

  const handleInsertSmsVariable = (displayToken: string) => {
    const input = smsTemplateInputRef.current
    const template = selectedConfig.sms_template
    const selectionStart = input?.selectionStart ?? template.length
    const selectionEnd = input?.selectionEnd ?? template.length
    const nextTemplate = `${template.slice(0, selectionStart)}${displayToken}${template.slice(selectionEnd)}`
    patchChannel(selectedChannel, { sms_template: nextTemplate })

    window.requestAnimationFrame(() => {
      input?.focus()
      const nextCursor = selectionStart + displayToken.length
      input?.setSelectionRange(nextCursor, nextCursor)
    })
  }

  const handleSave = async () => {
    setSaving(true)
    setError(null)
    try {
      const response = await api.setNotificationConfig(getBackendNotificationConfig(config))
      if (response.status === 'ok') {
        setSuccess('通知配置已保存')
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSaving(false)
    }
  }

  const handleTest = async () => {
    setTesting(true)
    setError(null)
    try {
      await api.setNotificationConfig(getBackendNotificationConfig(config))
      const response = await api.testNotificationChannel(selectedChannel)
      if (response.status === 'ok' && response.data) {
        if (response.data.success) {
          setSuccess(response.data.message)
        } else {
          setError(response.data.message)
        }
      } else {
        setError(response.message)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setTesting(false)
    }
  }

  const renderForwardSwitch = (channel: NotificationChannelKey) => (
    <Box display="flex" gap={2} flexWrap="wrap" mb={2.5}>
      <FormControlLabel
        control={
          <Switch
            checked={config[channel].forward_sms}
            onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel(channel, { forward_sms: event.target.checked })}
            disabled={!config[channel].enabled}
          />
        }
        label="转发短信"
      />
      {/*
      <FormControlLabel
        control={
          <Switch
            checked={config[channel].forward_calls}
            onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel(channel, { forward_calls: event.target.checked })}
            disabled={!config[channel].enabled}
          />
        }
        label="转发电话"
      />
      */}
    </Box>
  )

  const renderSmsTemplateEditor = (channel: NotificationChannelKey, jsonTemplate = false) => (
    <Box sx={{ mt: 3, pt: 2.5, borderTop: 1, borderColor: 'divider' }}>
      {renderForwardSwitch(channel)}
      <Typography variant="subtitle2" sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1.5 }}>
        {jsonTemplate ? 'Payload 模板' : '消息模板'}
        {jsonTemplate && <Chip label="JSON" size="small" variant="outlined" />}
      </Typography>
      <Box display="flex" alignItems="center" gap={1} flexWrap="wrap" mb={2.5}>
        <Typography variant="body2" color="text.secondary" sx={{ mr: 0.5 }}>
          短信变量：
        </Typography>
        {SMS_TEMPLATE_VARIABLES.map((variable) => (
          <Chip
            key={variable.displayToken}
            label={`+ ${variable.label}`}
            size="small"
            variant="outlined"
            clickable
            disabled={!config[channel].enabled}
            onClick={() => handleInsertSmsVariable(variable.displayToken)}
          />
        ))}
      </Box>
      <TextField
        fullWidth
        label={jsonTemplate ? '短信通知 Payload' : '短信通知模板'}
        inputRef={channel === selectedChannel ? smsTemplateInputRef : undefined}
        value={config[channel].sms_template}
        onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel(channel, { sms_template: event.target.value })}
        multiline
        rows={jsonTemplate ? 7 : 5}
        disabled={!config[channel].enabled}
        sx={{ mb: 2 }}
        InputProps={{
          sx: {
            fontFamily: 'monospace',
            fontSize: '0.85rem',
            '& textarea': {
              lineHeight: 1.75,
            },
          },
        }}
      />
      <Button
        size="small"
        variant="outlined"
        onClick={() => patchChannel(channel, {
          sms_template: channel === 'webhook' ? DEFAULT_SMS_DISPLAY_TEMPLATE : DEFAULT_PLAIN_SMS_DISPLAY_TEMPLATE,
        })}
        disabled={!config[channel].enabled}
      >
        重置为默认模板
      </Button>
    </Box>
  )

  const renderWebhookFields = () => (
    <>
      <TextField
        fullWidth
        label="Webhook URL"
        value={config.webhook.url}
        onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('webhook', { url: event.target.value })}
        placeholder="https://example.com/webhook"
        disabled={!config.webhook.enabled}
        sx={{ mb: 2 }}
      />
      <TextField
        fullWidth
        label="签名密钥"
        value={config.webhook.secret}
        onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('webhook', { secret: event.target.value })}
        type="password"
        disabled={!config.webhook.enabled}
        sx={{ mb: 2 }}
      />
      <Typography variant="subtitle2" gutterBottom>自定义请求头</Typography>
      <Box display="flex" gap={1} mb={1} flexWrap={{ xs: 'wrap', sm: 'nowrap' }}>
        <TextField
          size="small"
          label="Header Key"
          value={newHeaderKey}
          onChange={(event: ChangeEvent<HTMLInputElement>) => setNewHeaderKey(event.target.value)}
          disabled={!config.webhook.enabled}
          sx={{ flex: '1 1 180px' }}
        />
        <TextField
          size="small"
          label="Header Value"
          value={newHeaderValue}
          onChange={(event: ChangeEvent<HTMLInputElement>) => setNewHeaderValue(event.target.value)}
          disabled={!config.webhook.enabled}
          sx={{ flex: '1 1 180px' }}
        />
        <Tooltip title="添加请求头">
          <span>
            <IconButton
              color="primary"
              onClick={handleAddHeader}
              disabled={!config.webhook.enabled || !newHeaderKey.trim() || !newHeaderValue.trim()}
            >
              <Add />
            </IconButton>
          </span>
        </Tooltip>
      </Box>
      {Object.keys(config.webhook.headers).length > 0 && (
        <Box mb={2}>
          {Object.entries(config.webhook.headers).map(([key, value]) => (
            <Chip
              key={key}
              label={`${key}: ${value}`}
              onDelete={() => handleRemoveHeader(key)}
              deleteIcon={<DeleteOutline />}
              size="small"
              sx={{ mr: 1, mb: 1 }}
              disabled={!config.webhook.enabled}
            />
          ))}
        </Box>
      )}
      {renderSmsTemplateEditor('webhook', true)}
    </>
  )

  const renderBarkFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="Server URL"
          value={config.bark.server_url}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { server_url: event.target.value })}
          disabled={!config.bark.enabled}
        />
        <TextField
          label="Device Key"
          value={config.bark.device_key}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { device_key: event.target.value })}
          type="password"
          disabled={!config.bark.enabled}
        />
        <TextField
          label="标题模板"
          value={config.bark.title_template}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { title_template: event.target.value })}
          disabled={!config.bark.enabled}
        />
        <TextField
          label="分组"
          value={config.bark.group}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { group: event.target.value })}
          disabled={!config.bark.enabled}
        />
        <TextField
          label="铃声"
          value={config.bark.sound}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { sound: event.target.value })}
          disabled={!config.bark.enabled}
        />
        <TextField
          select
          label="推送等级"
          value={config.bark.level}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { level: event.target.value })}
          disabled={!config.bark.enabled}
        >
          <MenuItem value="">默认</MenuItem>
          <MenuItem value="active">active</MenuItem>
          <MenuItem value="timeSensitive">timeSensitive</MenuItem>
          <MenuItem value="passive">passive</MenuItem>
        </TextField>
      </Box>
      <Box display="flex" gap={2} mb={2} flexWrap="wrap">
        <FormControlLabel
          control={
            <Switch
              checked={config.bark.auto_copy}
              onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { auto_copy: event.target.checked })}
              disabled={!config.bark.enabled}
            />
          }
          label="自动复制"
        />
        <FormControlLabel
          control={
            <Switch
              checked={config.bark.save_history}
              onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('bark', { save_history: event.target.checked })}
              disabled={!config.bark.enabled}
            />
          }
          label="保存历史记录"
        />
      </Box>
      {renderSmsTemplateEditor('bark')}
    </>
  )

  const renderWecomAppFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="CorpID"
          value={config.wecom_app.corp_id}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { corp_id: event.target.value })}
          disabled={!config.wecom_app.enabled}
        />
        <TextField
          label="AgentID"
          value={config.wecom_app.agent_id}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { agent_id: event.target.value })}
          disabled={!config.wecom_app.enabled}
        />
        <TextField
          label="Secret"
          value={config.wecom_app.secret}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { secret: event.target.value })}
          type="password"
          disabled={!config.wecom_app.enabled}
        />
        <TextField
          label="ToUser"
          value={config.wecom_app.to_user}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { to_user: event.target.value })}
          disabled={!config.wecom_app.enabled}
        />
        <TextField
          label="ToParty"
          value={config.wecom_app.to_party}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { to_party: event.target.value })}
          disabled={!config.wecom_app.enabled}
        />
        <TextField
          label="ToTag"
          value={config.wecom_app.to_tag}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { to_tag: event.target.value })}
          disabled={!config.wecom_app.enabled}
        />
      </Box>
      <FormControlLabel
        sx={{ mb: 2 }}
        control={
          <Switch
            checked={config.wecom_app.safe}
            onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_app', { safe: event.target.checked })}
            disabled={!config.wecom_app.enabled}
          />
        }
        label="保密消息"
      />
      {renderSmsTemplateEditor('wecom_app')}
    </>
  )

  const renderWecomRobotFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="Webhook URL"
          value={config.wecom_robot.webhook_url}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_robot', { webhook_url: event.target.value })}
          disabled={!config.wecom_robot.enabled}
        />
        <TextField
          label="Webhook Key"
          value={config.wecom_robot.key}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('wecom_robot', { key: event.target.value })}
          type="password"
          disabled={!config.wecom_robot.enabled}
        />
      </Box>
      {renderSmsTemplateEditor('wecom_robot')}
    </>
  )

  const renderDingtalkRobotFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="Webhook URL"
          value={config.dingtalk_robot.webhook_url}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_robot', { webhook_url: event.target.value })}
          disabled={!config.dingtalk_robot.enabled}
        />
        <TextField
          label="Access Token"
          value={config.dingtalk_robot.access_token}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_robot', { access_token: event.target.value })}
          type="password"
          disabled={!config.dingtalk_robot.enabled}
        />
        <TextField
          label="加签 Secret"
          value={config.dingtalk_robot.secret}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_robot', { secret: event.target.value })}
          type="password"
          disabled={!config.dingtalk_robot.enabled}
        />
        <TextField
          label="At Mobiles"
          value={config.dingtalk_robot.at_mobiles}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_robot', { at_mobiles: event.target.value })}
          disabled={!config.dingtalk_robot.enabled}
        />
      </Box>
      <FormControlLabel
        sx={{ mb: 2 }}
        control={
          <Switch
            checked={config.dingtalk_robot.at_all}
            onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_robot', { at_all: event.target.checked })}
            disabled={!config.dingtalk_robot.enabled}
          />
        }
        label="@所有人"
      />
      {renderSmsTemplateEditor('dingtalk_robot')}
    </>
  )

  const renderDingtalkAppFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="AppKey"
          value={config.dingtalk_app.app_key}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_app', { app_key: event.target.value })}
          disabled={!config.dingtalk_app.enabled}
        />
        <TextField
          label="AppSecret"
          value={config.dingtalk_app.app_secret}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_app', { app_secret: event.target.value })}
          type="password"
          disabled={!config.dingtalk_app.enabled}
        />
        <TextField
          label="RobotCode"
          value={config.dingtalk_app.robot_code}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_app', { robot_code: event.target.value })}
          disabled={!config.dingtalk_app.enabled}
        />
        <TextField
          label="OpenConversationId"
          value={config.dingtalk_app.open_conversation_id}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_app', { open_conversation_id: event.target.value })}
          disabled={!config.dingtalk_app.enabled}
        />
        <TextField
          label="MsgKey"
          value={config.dingtalk_app.msg_key}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('dingtalk_app', { msg_key: event.target.value })}
          disabled={!config.dingtalk_app.enabled}
        />
      </Box>
      {renderSmsTemplateEditor('dingtalk_app')}
    </>
  )

  const renderFeishuRobotFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="Webhook URL"
          value={config.feishu_robot.webhook_url}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('feishu_robot', { webhook_url: event.target.value })}
          disabled={!config.feishu_robot.enabled}
        />
        <TextField
          label="Token"
          value={config.feishu_robot.token}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('feishu_robot', { token: event.target.value })}
          type="password"
          disabled={!config.feishu_robot.enabled}
        />
        <TextField
          label="加签 Secret"
          value={config.feishu_robot.secret}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('feishu_robot', { secret: event.target.value })}
          type="password"
          disabled={!config.feishu_robot.enabled}
        />
      </Box>
      {renderSmsTemplateEditor('feishu_robot')}
    </>
  )

  const renderTelegramFields = () => (
    <>
      <Box display="grid" gridTemplateColumns={{ xs: '1fr', md: '1fr 1fr' }} gap={2} mb={2}>
        <TextField
          label="Bot Token"
          value={config.telegram.bot_token}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('telegram', { bot_token: event.target.value })}
          type="password"
          disabled={!config.telegram.enabled}
        />
        <TextField
          label="Chat ID"
          value={config.telegram.chat_id}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('telegram', { chat_id: event.target.value })}
          disabled={!config.telegram.enabled}
        />
        <TextField
          select
          label="Parse Mode"
          value={config.telegram.parse_mode}
          onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('telegram', { parse_mode: event.target.value })}
          disabled={!config.telegram.enabled}
        >
          <MenuItem value="">无</MenuItem>
          <MenuItem value="MarkdownV2">MarkdownV2</MenuItem>
          <MenuItem value="HTML">HTML</MenuItem>
        </TextField>
      </Box>
      <FormControlLabel
        sx={{ mb: 2 }}
        control={
          <Switch
            checked={config.telegram.disable_web_page_preview}
            onChange={(event: ChangeEvent<HTMLInputElement>) => patchChannel('telegram', { disable_web_page_preview: event.target.checked })}
            disabled={!config.telegram.enabled}
          />
        }
        label="禁用链接预览"
      />
      {renderSmsTemplateEditor('telegram')}
    </>
  )

  const renderSelectedFields = () => {
    switch (selectedChannel) {
      case 'webhook':
        return renderWebhookFields()
      case 'bark':
        return renderBarkFields()
      case 'wecom_app':
        return renderWecomAppFields()
      case 'wecom_robot':
        return renderWecomRobotFields()
      case 'dingtalk_robot':
        return renderDingtalkRobotFields()
      case 'dingtalk_app':
        return renderDingtalkAppFields()
      case 'feishu_robot':
        return renderFeishuRobotFields()
      case 'telegram':
        return renderTelegramFields()
      default:
        return null
    }
  }

  const channelListContent = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box display="flex" gap={1} p={2} flexWrap="wrap">
        <Paper sx={{ p: 1, flex: 1, minWidth: 100, textAlign: 'center' }}>
          <Typography variant="h6" color="text.primary" fontWeight={600}>{CHANNEL_KEYS.length}</Typography>
          <Typography variant="caption" color="text.secondary">可用渠道</Typography>
        </Paper>
        <Paper sx={{ p: 1, flex: 1, minWidth: 100, textAlign: 'center' }}>
          <Typography variant="h6" color="primary" fontWeight={600}>{enabledCount}</Typography>
          <Typography variant="caption" color="text.secondary">已启用渠道</Typography>
        </Paper>
      </Box>

      <Box display="flex" alignItems="center" px={2} pb={1}>
        <Typography variant="subtitle1" fontWeight={600}>
          通知渠道
        </Typography>
      </Box>

      <Divider />

      <List sx={{ flex: 1, overflow: 'auto' }}>
        {CHANNEL_DEFS.map((channel) => {
          const Icon = channel.icon
          const enabled = config[channel.key].enabled
          return (
            <Box key={channel.key}>
              <ListItemButton
                onClick={() => handleSelectChannel(channel.key)}
                selected={selectedChannel === channel.key}
                sx={{ gap: 1.25, alignItems: 'center', py: 1.25 }}
              >
                <Avatar
                  sx={{
                    width: 36,
                    height: 36,
                    border: 1,
                    borderColor: enabled ? 'primary.main' : 'divider',
                    bgcolor: (theme: Theme) => enabled ? alpha(theme.palette.primary.main, 0.08) : theme.palette.action.hover,
                    color: enabled ? 'primary.main' : 'text.disabled',
                  }}
                >
                  <Icon fontSize="small" />
                </Avatar>
                <ListItemText
                  primary={
                    <Typography variant="body1" fontWeight={600} noWrap>{channel.label}</Typography>
                  }
                />
              </ListItemButton>
            </Box>
          )
        })}
      </List>
    </Box>
  )

  const configAreaContent = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box
        sx={{
          p: 2,
          borderBottom: 1,
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          gap: 1,
        }}
      >
        {isMobile && (
          <IconButton onClick={() => setShowMobileList(true)} edge="start">
            <ArrowBack />
          </IconButton>
        )}
        <Box minWidth={0} display="flex" alignItems="center" gap={1.25}>
          <Avatar
            sx={{
              width: 36,
              height: 36,
              border: 1,
              borderColor: selectedConfig.enabled ? 'primary.main' : 'divider',
              bgcolor: (theme: Theme) => selectedConfig.enabled ? alpha(theme.palette.primary.main, 0.08) : theme.palette.action.hover,
              color: selectedConfig.enabled ? 'primary.main' : 'text.disabled',
              flexShrink: 0,
            }}
          >
            <SelectedIcon fontSize="small" />
          </Avatar>
          <Typography variant="subtitle1" fontWeight={600} noWrap>{selectedDef.label}</Typography>
          <Divider orientation="vertical" flexItem sx={{ height: 24, alignSelf: 'center' }} />
          <Switch
            checked={selectedConfig.enabled}
            onChange={(event: ChangeEvent<HTMLInputElement>) => setChannelEnabled(selectedChannel, event.target.checked)}
            inputProps={{ 'aria-label': `启用 ${selectedDef.label}` }}
          />
        </Box>
        <Box flexGrow={1} />
        <Box display="flex" gap={1} justifyContent="flex-end" flexWrap="wrap">
          <Button
            variant="outlined"
            onClick={() => void handleTest()}
            disabled={testing || !isChannelConfigured(selectedChannel, config)}
            startIcon={testing ? <CircularProgress size={18} /> : <PlayArrow />}
            sx={{ height: 36, minWidth: 112, flexShrink: 0, whiteSpace: 'nowrap' }}
          >
            {testing ? '发送中...' : '发送测试'}
          </Button>
          <Button
            variant="contained"
            onClick={() => void handleSave()}
            disabled={saving}
            startIcon={saving ? <CircularProgress size={18} /> : <Save />}
            sx={{ height: 36, minWidth: 112, flexShrink: 0, whiteSpace: 'nowrap' }}
          >
            {saving ? '保存中...' : '保存配置'}
          </Button>
        </Box>
      </Box>

      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        {renderSelectedFields()}
      </Box>
    </Box>
  )

  if (loading && !config) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="60vh">
        <CircularProgress />
      </Box>
    )
  }

  return (
    <Box sx={{ height: 'calc(100vh - 140px)', minHeight: 560 }}>
      <Box display="flex" alignItems="center" gap={1} mb={2}>
        <NotificationsActive color="primary" />
        <Typography variant="h5" fontWeight={600}>
          通知中心
        </Typography>
      </Box>

      <ErrorSnackbar error={error} onClose={() => setError(null)} />
      <Snackbar
        open={!!success}
        autoHideDuration={3000}
        resumeHideDuration={3000}
        onClose={() => setSuccess(null)}
        anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
      >
        <Alert severity="info" variant="filled" onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      </Snackbar>

      <Card sx={{ height: 'calc(100% - 48px)' }}>
        <CardContent sx={{ height: '100%', p: 0, '&:last-child': { pb: 0 } }}>
          {isMobile ? (
            showMobileList ? channelListContent : configAreaContent
          ) : (
            <Box display="flex" height="100%">
              <Box
                sx={{
                  width: 288,
                  borderRight: 1,
                  borderColor: 'divider',
                  flexShrink: 0,
                }}
              >
                {channelListContent}
              </Box>
              <Box sx={{ flex: 1, minWidth: 0 }}>
                {configAreaContent}
              </Box>
            </Box>
          )}
        </CardContent>
      </Card>
    </Box>
  )
}
