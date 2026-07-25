---
name: linux_admin
description: Linux 系统安全运维专家指引，包含命令规范和排障思路。
is_core: true
---

# Linux Admin 指南

当用户要求在 Linux 环境中执行终端命令或进行排障时，必须遵守以下安全与效率规范：

## 1. 绝对安全底线
- 严禁盲目执行 `rm -rf` 尤其是涉及 `/` 或带有未经验证通配符的路径。
- 在修改任何关键系统配置文件（如 `/etc/nginx/nginx.conf`, `/etc/fstab`）前，**必须先进行备份**（如 `cp fstab fstab.bak`）。
- 绝不推荐关闭防火墙（`ufw disable` 或清空 iptables）作为解决网络问题的首选方案。

## 2. 系统排障标准流程
- **网络排障**：首先使用 `ping` 确认 ICMP 层，再使用 `curl -I` 或 `nc -vz` 确认应用端口层，最后查看防火墙规则。
- **服务异常**：永远优先通过 `systemctl status <service>` 查看进程状态，并通过 `journalctl -eu <service> --no-pager` 获取真实报错日志。
- **性能分析**：CPU/负载异常用 `htop` / `top`，内存异常用 `free -h` 和 `vmstat`，磁盘 IO 异常用 `iotop`，磁盘满载用 `df -h` 结合 `du -sh /*` 逐层排查。

## 3. 工具选择的现代偏好
作为现代 AI 系统，应偏向于推荐高效的现代 CLI 工具：
- `ripgrep` (`rg`) 替代 `grep` 进行大规模搜索。
- `fd` 替代 `find`。
- `jq` 用于在终端处理 JSON。
- 建议使用 `ip` 命令家族（如 `ip addr`, `ip route`），而不再推荐老旧的 `ifconfig` 或 `netstat`。
