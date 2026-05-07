<a href="https://github.com/3899/SimAdmin">
  <img src="https://socialify.git.ci/3899/SimAdmin/image?description=1&descriptionEditable=%E9%9D%A2%E5%90%91%20Debian%20%E5%B9%B3%E5%8F%B0%E6%94%AF%E6%8C%81%20SIM%2FeSIM%20%E8%9C%82%E7%AA%9D%E8%AE%BE%E5%A4%87%E7%9A%84%E5%BC%80%E6%BA%90%20Web%20%E7%AE%A1%E7%90%86%E7%B3%BB%E7%BB%9F&font=Source%20Code%20Pro&logo=https%3A%2F%2Fgithub.com%2F3899%2FSimAdmin%2Fblob%2Fmain%2Ffrontend%2Fpublic%2Fsimadmin-logo.svg%3Fraw%3Dtrue&name=1&owner=1&pattern=Floating%20Cogs&theme=Auto" alt="SimAdmin" />
</a>

<div align="center">
  <br/>

  <div>
    <a href="https://github.com/3899/SimAdmin/releases">
      <img 
        alt="Debian"
        src="https://img.shields.io/badge/Debian-%23D70A53?logo=debian&logoColor=white&style=flat-square" 
      />
    </a >
    <a href="./LICENSE">
      <img
        src="https://img.shields.io/github/license/3899/SimAdmin?style=flat-square"
      />
    </a >
    <a href="https://github.com/3899/SimAdmin/releases">
      <img
        src="https://img.shields.io/github/v/release/3899/SimAdmin?style=flat-square"
      />
    </a >
    <a href="https://github.com/3899/SimAdmin/releases">
      <img
        src="https://img.shields.io/github/downloads/3899/SimAdmin/total?style=flat-square"
      />  
    </a >
  </div>

  <br/>

  <picture>
    <img src="./static/Dashboard.png" width="100%" alt="Dashboard" />
	<br/><br/>
	<img src="./static/Device_Information.png" width="100%" alt="Device_Information" />
	<br/><br/>
	<img src="./static/SMS.png" width="100%" alt="SMS" />
	<br/><br/>
	<img src="./static/Notification.png" width="100%" alt="Notification" />
	<br/><br/>
	<img src="./static/OTA.png" width="100%" alt="OTA" />
	<br/><br/>
  </picture>
  
</div>

# SimAdmin - SIM/eSIM 中枢

SimAdmin 是一套面向 Debian 蜂窝 CPE、随身 WiFi、软路由类设备的 SIM/eSIM、蜂窝网络、短信和系统状态管理系统。

当前项目由 Rust 后端和 React 前端组成：

- 后端：Rust + Axum + zbus，主要通过 ModemManager D-Bus 接口管理 modem，并在部分场景使用 `mmcli`、`qmicli` 或 AT 直连兜底。
- 前端：React + Vite + Material UI，提供仪表盘、设备信息、网络管理、短信、通知配置和 OTA 更新页面。
- 部署形态：后端二进制同进程托管前端 SPA，默认安装到 `/opt/simadmin`，通过 systemd 运行。

健康检查整体按支持 ModemManager 的 Linux 蜂窝设备组织，不同 modem 固件、内核、ModemManager 版本暴露的能力不同，具体功能以实际设备为准。

## 免责声明

本项目会直接操作蜂窝 modem、SIM 注册、数据拨号、APN、频段、飞行模式、iptables、NetworkManager、systemd 服务、系统重启和 OTA 文件替换。

请仅在你拥有控制权的设备上使用。错误配置可能导致断网、无法注册网络、SIM 漫游计费、设备需要手动恢复，甚至 OTA 后服务无法启动。任何使用本项目造成的后果由使用者自行承担。

部分接口受硬件和 ModemManager 能力限制：

- 频段锁定依赖 ModemManager 暴露的 `SupportedBands` / `CurrentBands` / `SetCurrentBands`。
- 小区锁定当前为后端内存态展示，不会下发真实硬件锁小区命令。

## 开源协议声明

本项目采用 GNU General Public License v3.0 (GPLv3) 开源协议。

你可以：

- 自由使用、研究、修改本软件。
- 分发本软件副本。
- 分发修改后的版本。

但你必须：

1. 保留版权声明和许可证声明。
2. 分发本软件或修改版本时，以 GPLv3 协议公开完整源代码。
3. 基于本项目的衍生作品继续使用 GPLv3 协议。
4. 明确标注修改内容和修改日期。
5. 分发时附带完整 GPLv3 许可证文本。

严禁将本项目或其衍生版本闭源后作为专有软件分发。

## 快速开始

### 项目结构

```text
.
├── backend/          # Rust + Axum 后端，ModemManager、SQLite、OTA、通知、系统接口
├── frontend/         # React + Vite + MUI 前端
├── bruno-api/        # Bruno API 调试集合
├── scripts/          # 构建、部署、systemd、modem 恢复脚本
├── install_latest.sh # 设备侧一键安装 / 升级脚本
├── uninstall.sh      # 设备侧一键卸载脚本
├── VERSION           # 项目版本号
└── LICENSE           # GPLv3 许可证
```

### 前端开发

```bash
cd frontend
pnpm install
pnpm dev
```

构建前端：

```bash
cd frontend
pnpm run build
```

前端构建产物输出到 `frontend/dist/`，部署后会复制为 `/opt/simadmin/www/`。

### 后端开发

```bash
cd backend
cargo check
cargo run -- --host :: --port 3000
```

参数和环境变量：

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| `--host` / `-H` | `HOST` | `::` | 监听地址，默认双栈 IPv4/IPv6 |
| `--port` / `-p` | `PORT` | `3000` | HTTP 监听端口 |

在普通开发机上运行后端时，如果没有 system D-Bus、ModemManager 或 modem，硬件相关接口会返回错误，这是预期行为。

### 构建完整 OTA 包

```bash
./scripts/build.sh
```

常用选项：

```bash
./scripts/build.sh --backend-only
./scripts/build.sh --frontend-only
./scripts/build.sh --no-upx
./scripts/build.sh --no-ota
```

构建脚本会：

- 同步 `VERSION` 到 `backend/Cargo.toml` 和 `frontend/package.json`。
- 构建前端到 `frontend/dist/`。
- 交叉编译后端到 `backend/target/aarch64-unknown-linux-musl/release/simadmin`。
- 可选使用 UPX 压缩后端二进制。
- 生成 `release/simadmin_<version>.tar.gz` OTA 包。

### 通过 ADB 部署

```bash
./scripts/deploy.sh
```

常用选项：

```bash
./scripts/deploy.sh --backend-only
./scripts/deploy.sh --frontend-only
./scripts/deploy.sh --no-restart
./scripts/deploy.sh --target=/opt/simadmin
```

### 设备侧一键安装 / 升级

在目标设备上以 root 执行：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/install_latest.sh | sh
```

可选环境变量：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/install_latest.sh \
  | REPO=3899/SimAdmin INSTALL_DIR=/opt/simadmin SERVICE_NAME=simadmin sh
```

脚本会：

- 从 GitHub Release 下载 `simadmin_latest.tar.gz` 或指定版本包。
- 安装后端二进制到 `/opt/simadmin/simadmin`。
- 安装前端到 `/opt/simadmin/www`。
- 安装并启用 `simadmin.service`。
- 安装并启用 `simadmin-modem-recovery.service`。
- 配置 NetworkManager 忽略 `wwan*` 接口，避免与 SimAdmin 抢占蜂窝连接管理。

### 设备侧一键卸载

默认彻底卸载，删除服务、程序文件、前端文件、OTA 临时目录、NetworkManager 配置以及用户数据：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh | sh
```

如需保留短信数据库和配置文件：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh \
  | sh -s -- --keep-user-data
```

自定义安装路径或服务名时，需要和安装时保持一致：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh \
  | INSTALL_DIR=/opt/simadmin SERVICE_NAME=simadmin sh -s -- --keep-user-data
```

可选参数：

| 参数 | 说明 |
|------|------|
| `--purge` | 删除全部 SimAdmin 文件和用户数据，默认行为 |
| `--keep-user-data` | 保留 `/opt/simadmin/data.db`、SQLite sidecar 文件和配置文件 |
| `--install-dir PATH` | 指定安装目录，默认 `/opt/simadmin` |
| `--service-name NAME` | 指定主服务名，默认 `simadmin` |

脚本会：

- 停止并禁用 `simadmin.service`。
- 停止并禁用 `simadmin-modem-recovery.service`。
- 删除 systemd 单元文件并执行 `daemon-reload` / `reset-failed`。
- 删除 `/usr/local/bin/simadmin-modem-recovery.sh`。
- 删除 `/etc/NetworkManager/conf.d/99-simadmin-unmanaged-modem.conf`，并在 NetworkManager 运行时重启它。
- 删除 `/tmp/ota_staging`。
- 默认删除 `/opt/simadmin` 和 `/data/config.json`；使用 `--keep-user-data` 时保留用户数据。

## 运行环境

### 目标设备要求

- Linux / Debian 系统。
- systemd。
- root 权限。
- system D-Bus。
- ModemManager 和 `mmcli`。
- 建议安装 `qmicli`，用于基站信息兜底读取。
- `iptables` / `ip6tables`，用于连接看门狗清理异常规则。
- `tar`，OTA 安装必需。
- `unzip`，仅在上传 zip 格式 OTA 包时需要。

### 安装路径

| 路径 | 说明 |
|------|------|
| `/opt/simadmin/simadmin` | 后端二进制 |
| `/opt/simadmin/www/` | 前端静态文件 |
| `/opt/simadmin/data.db` | SQLite 数据库，保存短信等业务数据 |
| `/opt/simadmin/meta.json` | 当前安装包元数据 |
| `/data/config.json` | 优先使用的持久化配置文件 |
| `/opt/simadmin/config.json` | `/data` 不存在时的配置文件回退路径 |
| `/tmp/ota_staging` | OTA 上传后的临时验证目录 |
| `/etc/systemd/system/simadmin.service` | 主服务 |
| `/etc/systemd/system/simadmin-modem-recovery.service` | 开机 modem 自检恢复服务 |
| `/usr/local/bin/simadmin-modem-recovery.sh` | 开机 modem 自检恢复脚本 |
| `/etc/NetworkManager/conf.d/99-simadmin-unmanaged-modem.conf` | NetworkManager 忽略 `wwan*` 的配置 |

### systemd 服务

主服务单元位于 `scripts/simadmin.service`，默认：

- `WorkingDirectory=/opt/simadmin`
- `ExecStart=/opt/simadmin/simadmin`
- `Restart=always`
- `DBUS_SYSTEM_BUS_ADDRESS=unix:path=/var/run/dbus/system_bus_socket`

查看状态：

```bash
systemctl status simadmin --no-pager
journalctl -u simadmin -f
```

## 核心功能

### Web 管理页面

| 页面 | 路由 | 说明 |
|------|------|------|
| 仪表盘 | `/` | 在线状态、运营商、信号、网络延迟、数据/漫游/飞行模式快捷开关、系统资源、温度、流量 |
| 设备信息 | `/device` | IMEI、厂商、型号、固件、SIM、系统信息 |
| 网络管理 | `/network` | 网络注册、服务小区和邻区、运营商扫描、APN、射频模式、频段锁定、小区锁定状态 |
| 短信管理 | `/sms` | 发送短信、短信列表、会话、统计、清空 |
| 通知中心 | `/notifications` | 多渠道通知配置、短信转发模板、测试发送 |
| 系统配置 | `/config` | 设备操作、数据连接、漫游、飞行模式、基带重启、服务重启、系统重启等 |
| OTA 更新 | `/ota` | 上传 OTA 包、在线获取 Release、验证、应用或取消更新 |

### 后端能力

- 设备信息、SIM 信息、网络注册信息读取。
- 数据连接开关和漫游策略持久化。
- 飞行模式控制。
- 基带重启流程和进度查询。
- 数据连接 watchdog，每 15 秒检查连接状态、iptables 规则和 modem 可用性。
- ModemManager 丢失时触发 `mmcli --scan-modems`，连续失败后重启 ModemManager。
- NetworkManager `wwan*` unmanaged 配置。
- 短信发送、接收监听、SQLite 持久化和多渠道通知转发。
- APN 列表读取和 APN 修改。
- 运营商列表、扫描、手动注册、自动注册。
- OTA 上传、在线下载、校验、替换二进制和前端资源。

## 🚀 版本更新记录

### 📌 v1.0.1

#### ✨ 新增功能

- 新增“通知中心”多渠道通知配置能力。
  - 通知配置从单一 Webhook 扩展为多渠道：
    - Webhook
    - Bark
    - 企业微信应用消息
    - 企业微信群机器人
    - 钉钉群自定义机器人
    - 钉钉企业内机器人
    - 飞书机器人
    - Telegram 机器人
  - 新增通知中心后端接口：
    - `GET /api/notifications/config`
    - `POST /api/notifications/config`
    - `POST /api/notifications/test/{channel}`
  - 新增各通知渠道的测试发送能力，可按渠道发送模拟短信测试。
- 新增短信批量管理能力。
  - 删除短信：
    - 删除单条短信
    - 删除整个短信对话
    - 批量删除短信或多个对话
  - 新增短信删除相关后端接口：
    - `DELETE /api/sms/message/{id}`
    - `DELETE /api/sms/conversation/{phone_number}`
    - `POST /api/sms/batch-delete`
- 新增设备侧一键卸载脚本 `uninstall.sh`，支持彻底卸载和保留用户数据卸载。

#### 💫 体验优化

- Dashboard 仪表盘加载性能优化。
- 通知中心页面重构为左右分栏布局：
  - 左侧展示通知渠道列表和启用统计
  - 右侧展示当前渠道配置表单
- 通知中心支持按渠道独立配置启用状态、短信转发、模板和渠道专属参数。
- 通知模板编辑支持中文变量标签，前端显示中文变量，保存时映射为后端变量。
- 通知渠道测试按钮会先保存当前配置，再执行测试，减少“配置未保存导致测试不生效”的误操作。
- 短信页面优化为更完整的对话管理体验：
  - 对话列表支持删除按钮
  - 聊天消息支持单条删除
  - 支持批量管理，有清晰的选择数量提示
  - 删除这种不可逆的危险操作增加二次确认
- 短信列表加载数量提升，减少历史短信遗漏。

#### 📚 接口与文档更新

- README 中 Webhook 章节改为“通知中心”，补充多渠道通知能力说明。
- Bruno 文档同步更新通知中心接口说明。
- README 新增卸载脚本使用说明、参数说明和清理范围说明。
- API 类型定义新增完整通知中心配置类型，包括各渠道配置结构。

## ModemManager D-Bus 接口

当前实现以 ModemManager 为主。

### 核心接口

| 接口 | 说明 |
|------|------|
| `org.freedesktop.ModemManager1` | ModemManager 根服务 |
| `org.freedesktop.ModemManager1.Modem` | Modem 状态、开关、模式、频段 |
| `org.freedesktop.ModemManager1.Modem.Modem3gpp` | 运营商、注册、扫描 |
| `org.freedesktop.ModemManager1.Modem.Simple` | 简化连接和断开 |
| `org.freedesktop.ModemManager1.Modem.Messaging` | 短信发送和接收 |
| `org.freedesktop.ModemManager1.Sim` | SIM 属性 |
| `org.freedesktop.ModemManager1.Bearer` | 数据连接 bearer |

### 常用调试命令

```bash
# 查看 modem 列表
mmcli -L

# 查看 modem 详情
mmcli -m any

# 查看注册和连接简要状态
mmcli -m any --simple-status

# 查看 3GPP 定位信息
mmcli -m any --location-get

# 查看信号指标
mmcli -m any --signal-get

# 发送 AT 指令
mmcli -m any --command='AT+CGSN'
```

### D-Bus 监控

```bash
# 监听 ModemManager 信号
dbus-monitor --system "sender='org.freedesktop.ModemManager1'"

# 查看 modem 0 暴露的接口
busctl introspect org.freedesktop.ModemManager1 /org/freedesktop/ModemManager1/Modem/0
```

## 频段与小区控制

### 射频模式

`/api/radio-mode` 支持：

| 值 | 说明 |
|----|------|
| `auto` | LTE/NR 自动 |
| `lte` | LTE only |
| `nr` | NR only |

实际是否可切换取决于 modem 的 `SupportedModes`。

### 频段锁定

频段锁定通过 ModemManager 的 `SetCurrentBands` 实现。API 使用用户熟悉的物理频段号，后端内部转换为 ModemManager band id：

- LTE：`30 + Bn`
- NR：`300 + Nn`

示例，锁定 LTE B1 + B3：

```json
{
  "lte_fdd_bands": [1, 3],
  "lte_tdd_bands": [],
  "nr_fdd_bands": [],
  "nr_tdd_bands": []
}
```

示例，锁定 NR N78：

```json
{
  "lte_fdd_bands": [],
  "lte_tdd_bands": [],
  "nr_fdd_bands": [],
  "nr_tdd_bands": [78]
}
```

示例，混合锁定 LTE 和 NR：

```json
{
  "lte_fdd_bands": [1, 3],
  "lte_tdd_bands": [38, 40, 41],
  "nr_fdd_bands": [],
  "nr_tdd_bands": [78, 79]
}
```

解锁所有频段时传空数组：

```json
{
  "lte_fdd_bands": [],
  "lte_tdd_bands": [],
  "nr_fdd_bands": [],
  "nr_tdd_bands": []
}
```

后端会与 modem 的 `SupportedBands` 取交集。如果所选频段不被当前 modem 支持，会返回错误。

### 小区锁定

当前 `/api/cell-lock` 只维护内存状态，支持记录 LTE 和 NR 的 ARFCN/PCI：

```json
{
  "rat": 16,
  "enable": true,
  "pci": 123,
  "arfcn": 627264
}
```

`rat=12` 表示 LTE，`rat=16` 表示 NR。这个接口当前不会下发真实 AT 或 QMI 小区锁定命令。

## API 接口文档

所有主要业务接口返回统一 JSON 包装：

```json
{
  "status": "ok",
  "message": "Success",
  "data": {}
}
```

错误响应：

```json
{
  "status": "error",
  "message": "错误信息"
}
```

### 基础信息

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/health` | GET | 健康检查，返回平台和版本 |
| `/api/device` | GET | 设备信息 |
| `/api/sim` | GET | SIM 卡信息 |

### 网络状态

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/network` | GET | 网络注册信息、运营商、信号百分比 |
| `/api/cells` | GET | 服务小区和邻区信息 |
| `/api/cell-monitor/start` | POST | 启动小区监控 |
| `/api/cell-monitor/stop` | POST | 停止小区监控 |
| `/api/network/interfaces` | GET | 网络接口、IP、流量统计 |
| `/api/network/signal-strength` | GET | 信号强度 |
| `/api/location/cell-info` | GET | 基站定位参数 |
| `/api/network/operators` | GET | 当前和已知运营商 |
| `/api/network/operators/scan` | GET | 扫描可用运营商，可能耗时较长 |
| `/api/network/register-manual` | POST | 手动注册运营商 |
| `/api/network/register-auto` | POST | 自动注册运营商 |
| `/api/connectivity` | GET | IPv4 / IPv6 连通性检测 |

### 模块控制

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/data` | GET/POST | 数据连接状态和开关 |
| `/api/roaming` | GET/POST | 漫游策略和当前漫游状态 |
| `/api/airplane-mode` | GET/POST | 飞行模式 |
| `/api/radio-mode` | GET/POST | 射频模式 |
| `/api/band-lock` | GET/POST | 频段锁定 |
| `/api/cell-lock` | GET/POST | 小区锁定内存状态 |
| `/api/cell-lock/unlock-all` | POST | 清空小区锁定状态 |
| `/api/apn` | GET/POST | APN 列表和配置 |
| `/api/baseband/restart` | POST | 重启基带并尝试恢复网络 |
| `/api/baseband/restart/status` | GET | 基带重启进度 |

### 短信功能

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/sms/send` | POST | 发送短信 |
| `/api/sms/list` | GET | 短信列表，支持 `limit` / `offset` |
| `/api/sms/conversation` | GET | 指定号码会话，参数 `phone_number` |
| `/api/sms/stats` | GET | 短信统计 |
| `/api/sms/clear` | POST | 清空短信记录 |
| `/api/sms/message/{id}` | DELETE | 删除单条短信 |
| `/api/sms/conversation/{phone_number}` | DELETE | 删除指定号码会话 |
| `/api/sms/batch-delete` | POST | 批量删除短信或多个会话 |

### 系统信息

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/stats` | GET | 网速、内存、磁盘、CPU、运行时间、系统信息、温度 |
| `/api/stats/cpu` | GET | CPU 详细信息 |
| `/api/system/reboot` | POST | 系统重启 |
| `/api/service/restart` | POST | 重启 SimAdmin 服务 |

### 通知中心

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/notifications/config` | GET/POST | 多渠道通知配置 |
| `/api/notifications/test/{channel}` | POST | 测试指定通知渠道 |

通知配置支持：

- Webhook。
- Bark。
- 企业微信应用消息。
- 企业微信群机器人。
- 钉钉群自定义机器人。
- 钉钉企业内机器人。
- 飞书机器人。
- Telegram 机器人。
- `sms_template` 模板。

模板变量包括短信号码、内容、方向、状态和时间等字段。

### OTA 更新

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/ota/status` | GET | 当前版本和待安装更新 |
| `/api/ota/upload` | POST | 上传 OTA 包，最大 50 MB |
| `/api/ota/latest-release` | POST | 查询 GitHub latest release |
| `/api/ota/online-prepare` | POST | 在线下载 latest release 中的 OTA 包并验证 |
| `/api/ota/apply` | POST | 应用待安装更新 |
| `/api/ota/cancel` | POST | 取消待安装更新 |

OTA 包结构：

```text
meta.json
simadmin
www/
```

`meta.json` 示例：

```json
{
  "version": "1.0.1",
  "commit": "abcdef0",
  "build_time": "2026-05-06T00:00:00Z",
  "binary_md5": "md5-of-binary",
  "frontend_md5": "md5-of-www",
  "arch": "aarch64-unknown-linux-musl"
}
```

当前验证逻辑会检查：

- OTA 包中存在 `simadmin`。
- OTA 包中存在 `www/`。
- 二进制 MD5 匹配。
- 架构为 `aarch64-unknown-linux-musl`。
- 版本号是否高于当前版本会作为提示字段返回，不是唯一安装条件。

## 开发指南

### D-Bus 操作序列化

会改变 modem 状态的操作应通过 `with_serial` 串行执行，避免 ModemManager 或底层设备出现并发冲突：

```rust
use crate::serial::with_serial;

pub async fn set_some_modem_state(conn: &Connection) -> zbus::Result<()> {
    with_serial(async {
        // D-Bus / modem operation
        Ok(())
    }).await
}
```

### 前后端契约

- 后端模型位于 `backend/src/models.rs`。
- 前端类型位于 `frontend/src/api/contracts.ts`。
- 前端 API 封装位于 `frontend/src/api/current.ts`。
- 路由集中在 `backend/src/main.rs` 和 `frontend/src/App.tsx`。

新增接口时建议同步修改：

1. `backend/src/models.rs`
2. `backend/src/handlers.rs`
3. `backend/src/main.rs`
4. `frontend/src/api/contracts.ts`
5. `frontend/src/api/current.ts`
6. 对应页面或 hook
7. `bruno-api/` 调试请求

### 数据持久化

SQLite 数据库保存：

- `sms_messages`：短信收发记录。

配置文件保存：

- 通知中心配置。
- 是否允许漫游。
- 数据连接是否由用户启用。

### 版本注入

`backend/build.rs` 会在编译期注入：

- `APP_VERSION`
- `GIT_BRANCH`
- `GIT_COMMIT`

其中版本号来自根目录 `VERSION`。

### Bruno API 集合

`bruno-api/` 是 Bruno 调试集合，默认按 `http://IP:3000` 组织请求。导入 Bruno 后可批量验证后端接口。

## 依赖

### 后端

- `axum 0.8`：HTTP 服务。
- `tokio 1.48`：异步运行时。
- `zbus 5`：D-Bus 客户端。
- `rusqlite 0.32`：SQLite。
- `tower-http 0.6`：CORS 和静态文件相关能力。
- `reqwest 0.12`：在线 OTA Release 查询和下载。
- `clap 4.5`：命令行参数。
- `tracing`：日志。

### 前端

- `react 19`
- `vite 7`
- `@mui/material 7`
- `@mui/icons-material`
- `@mui/x-charts`
- `@mui/x-data-grid`
- `@tanstack/react-query`
- `swr`
- `react-router-dom 7`

### 目标设备命令

- `ModemManager`
- `mmcli`
- `qmicli`
- `iptables` / `ip6tables`
- `systemctl`
- `tar`
- `unzip`
- `curl`

## license 许可证

GNU General Public License v3.0

## 🎖️ 鸣谢

> 本项目基于开源项目 `1orz/project-cpe` 进行深度重构适配和二次开发，在此向原作者、贡献者及开源社区致以诚挚谢意。
