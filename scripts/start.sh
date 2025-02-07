set -euo pipefail

MAX_RESTARTS=3    # Maximum allowed restarts before giving up
RESTART_DELAY=10   # Delay (seconds) before restarting
RESET_TIME=30     # Time (seconds) to reset the restart counter

restart_count=0
last_restart_time=$(date +%s)

while true; do
  ./xailyser-server
  exit_code=$?

  current_time=$(date +%s)
  elapsed_time=$((current_time - last_restart_time))

  # Reset restart count if enough time has passed
  if [[ $elapsed_time -ge $RESET_TIME ]]; then
      restart_count=0
  fi

  if [[ $exit_code -eq 42 ]]; then
    echo "Got restart code from the program!"
    if [[ $restart_count -ge $MAX_RESTARTS ]]; then
        echo "Reached max restart attempts ($MAX_RESTARTS). Exiting."
        exit 1
    fi

    restart_count=$((restart_count + 1))
    last_restart_time=$current_time

    echo "Restarting program... (Attempt $restart_count/$MAX_RESTARTS)"
    sleep $RESTART_DELAY
  else
    echo "Program exited normally. No restart."
    exit 0
  fi
done
