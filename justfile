# Load .env without requiring it to be exported into the shell environment
set dotenv-load := true

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log := "echo 'INFO -'"

# ---------------------------------------------------------------------------
# Default: show available recipes
# ---------------------------------------------------------------------------

default:
    @just --list

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

build:
    cargo build --release
    @{{log}} "Building new binary locally complete"

# ---------------------------------------------------------------------------
# Copy binary to remote host
# ---------------------------------------------------------------------------

copy:
    sftp -i {{env_var("SSH_KEY_PATH")}} {{env_var("REMOTE_USER")}}@{{env_var("REMOTE_HOST")}} <<'EOF'
    put target/release/sg_assistant sg_assist/target/release/sg_assistant
    EOF
    @{{log}} "Copy to remote host complete"

# ---------------------------------------------------------------------------
# Restart service on remote host
# ---------------------------------------------------------------------------

deploy:
    ssh -i {{env_var("SSH_KEY_PATH")}} {{env_var("REMOTE_USER")}}@{{env_var("REMOTE_HOST")}} \
        'cd sg_assist && docker compose up --build -d && sleep 5 && docker compose logs --tail=5'
    @{{log}} "Update on remote host complete"

# ---------------------------------------------------------------------------
# Full pipeline: build → copy → deploy
# ---------------------------------------------------------------------------

release: build copy deploy
    @{{log}} "Full release complete"

# ---------------------------------------------------------------------------
# Tail live logs from remote
# ---------------------------------------------------------------------------

logs:
    ssh -i {{env_var("SSH_KEY_PATH")}} {{env_var("REMOTE_USER")}}@{{env_var("REMOTE_HOST")}} \
        'cd sg_assist && docker compose logs --follow --tail=50'

# ---------------------------------------------------------------------------
# Open a shell on the remote host
# ---------------------------------------------------------------------------

ssh:
    ssh -i {{env_var("SSH_KEY_PATH")}} {{env_var("REMOTE_USER")}}@{{env_var("REMOTE_HOST")}}
