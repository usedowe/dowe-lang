use crate::access::DeployAccess;
use crate::cloud;
use crate::embedded::{
    SSH_TRAILER_MAGIC, encode_embedded_payload, materialize_application, read_embedded_payload,
    set_executable, validate_access_metadata, validate_client_environment,
};
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{DeployEnvironment, DeployTarget};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_RELEASE_BASE_URL: &str = "https://get.dowe.dev";
const MAX_ARCHIVE_SIZE: u64 = 256 * 1024 * 1024;
const MAX_RUNTIME_SIZE: u64 = 512 * 1024 * 1024;
const REMOTE_SCRIPT: &str = r#"set -eu
action=$1
run_user=$2
case "$action" in
  preflight) ;;
  install)
    upload=$3
    env_upload=$4
    service=$5
    binary=$6
    cleanup() {
      rm -f "$upload"
      rm -f "$env_upload"
    }
    trap cleanup EXIT HUP INT TERM
    ;;
  *) echo "SSH deploy action is invalid" >&2; exit 1 ;;
esac
if [ ! -r /etc/os-release ]; then
  echo "SSH deploy requires Debian or Ubuntu" >&2
  exit 1
fi
. /etc/os-release
case "${ID:-}" in
  debian|ubuntu) ;;
  *) echo "SSH deploy requires Debian or Ubuntu" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) ;;
  *) echo "SSH deploy requires a Linux amd64 host" >&2; exit 1 ;;
esac
command -v systemctl >/dev/null 2>&1 || { echo "SSH deploy requires systemd" >&2; exit 1; }
id "$run_user" >/dev/null 2>&1 || { echo "SSH deploy user does not exist" >&2; exit 1; }
if [ "$(id -u)" -eq 0 ]; then
  as_root() {
    "$@"
  }
else
  command -v sudo >/dev/null 2>&1 || { echo "SSH deploy requires root or sudo" >&2; exit 1; }
  sudo -v
  as_root() {
    sudo "$@"
  }
fi
if [ "$action" = preflight ]; then
  exit 0
fi
install_dir="/opt/dowe/$service"
unit="/etc/systemd/system/$service.service"
group=$(id -gn "$run_user")
as_root install -d -m 0755 -o root -g root "$install_dir"
as_root install -m 0755 -o root -g root "$upload" "$install_dir/$binary"
unit_file=$(mktemp)
trap 'rm -f "$unit_file"; cleanup' EXIT HUP INT TERM
printf '%s\n' \
  '[Unit]' \
  "Description=Dowe SSH deployment $service" \
  'After=network-online.target' \
  'Wants=network-online.target' \
  '' \
  '[Service]' \
  'Type=simple' \
  "User=$run_user" \
  "Group=$group" \
  "WorkingDirectory=$install_dir" \
  "EnvironmentFile=-/etc/dowe/$service.env" \
  "Environment=DOWE_SSH_APP_ROOT=/var/lib/dowe/$service/app" \
  "ExecStart=$install_dir/$binary" \
  'Restart=always' \
  'RestartSec=3' \
  'NoNewPrivileges=true' \
  'PrivateTmp=true' \
  'ProtectSystem=strict' \
  'ProtectHome=true' \
  "ReadWritePaths=/var/lib/dowe/$service" \
  '' \
  '[Install]' \
  'WantedBy=multi-user.target' > "$unit_file"
as_root install -d -m 0755 /etc/dowe
as_root install -m 0600 -o root -g root "$env_upload" "/etc/dowe/$service.env"
as_root install -d -m 0755 -o root -g root /var/lib/dowe "/var/lib/dowe/$service"
as_root install -d -m 0755 -o "$run_user" -g "$group" "/var/lib/dowe/$service/app"
as_root install -m 0644 -o root -g root "$unit_file" "$unit"
as_root systemctl daemon-reload
as_root systemctl enable --now "$service.service"
as_root systemctl --no-pager --full status "$service.service"
"#;


include!("ssh_types_and_destination.rs");
include!("ssh_generation_and_apply.rs");
include!("ssh_runtime_download.rs");
include!("ssh_validation.rs");
include!("ssh_tests.rs");
