#!/bin/bash
# Install agent-pipeline systemd service

set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TOOL_DIR="$SCRIPT_DIR"
REPO_ROOT="$(cd -- "$TOOL_DIR/../.." && pwd)"
STATE_DIR="$TOOL_DIR/.agents"
SERVICE_TEMPLATE="$TOOL_DIR/agent-pipeline.service"

echo "Installing agent-pipeline systemd user service..."

# Create bin directory if needed
mkdir -p ~/.local/bin

# Create wrapper script
cat > ~/.local/bin/ash-pipeline << 'EOF'
#!/bin/bash
SCRIPT_DIR="__TOOL_DIR__"
export PYTHONPATH="$SCRIPT_DIR/src"
export AGENT_PIPELINE_WORKSPACE_ROOT="__REPO_ROOT__"
export AGENT_PIPELINE_BASE_DIR="__STATE_DIR__"
exec python3 "$SCRIPT_DIR/src/ash_pipeline.py" "$@"
EOF

python3 - <<PY
from pathlib import Path

wrapper_path = Path.home() / ".local/bin/ash-pipeline"
content = wrapper_path.read_text()
content = content.replace("__TOOL_DIR__", ${TOOL_DIR@Q})
content = content.replace("__REPO_ROOT__", ${REPO_ROOT@Q})
content = content.replace("__STATE_DIR__", ${STATE_DIR@Q})
wrapper_path.write_text(content)
PY

chmod +x ~/.local/bin/ash-pipeline

# Install systemd service
mkdir -p ~/.config/systemd/user
python3 - <<PY
from pathlib import Path

template = Path(${SERVICE_TEMPLATE@Q}).read_text()
rendered = (
	template.replace("@TOOL_DIR@", ${TOOL_DIR@Q})
	.replace("@REPO_ROOT@", ${REPO_ROOT@Q})
	.replace("@STATE_DIR@", ${STATE_DIR@Q})
)
(Path.home() / ".config/systemd/user/agent-pipeline.service").write_text(rendered)
PY

# Reload systemd
systemctl --user daemon-reload

echo "Service installed. To start:"
echo "  systemctl --user enable --now agent-pipeline"
echo ""
echo "To check status:"
echo "  systemctl --user status agent-pipeline"
echo "  journalctl --user -u agent-pipeline -f"
