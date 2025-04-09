$MAX_RESTARTS = 3       # Maximum allowed restarts before giving up
$RESTART_DELAY = 5      # Delay (seconds) before restarting
$RESET_TIME = 30        # Time (seconds) to reset the restart counter

$restart_count = 0
$last_restart_time = [DateTime]::UtcNow

Write-Host "Starting the script..."

while ($true) {
    & .\server.exe
    $exit_code = $LASTEXITCODE

    $current_time = [DateTime]::UtcNow
    $elapsed_time = ($current_time - $last_restart_time).TotalSeconds

    Write-Host "Process exited with code: $exit_code"
    Write-Host "Elapsed time since last restart: $elapsed_time sec"

    # Reset restart count if enough time has passed
    if ($elapsed_time -ge $RESET_TIME) {
        $restart_count = 0
    }

    if ($exit_code -eq 42) {
        Write-Host "Got restart code from the program!"
        if ($restart_count -ge $MAX_RESTARTS) {
            Write-Host "Reached max restart attempts ($MAX_RESTARTS). Exiting."
            exit 1
        }

        $restart_count++
        $last_restart_time = $current_time

        Write-Host "Restarting program... (Attempt $restart_count/$MAX_RESTARTS)"
        Start-Sleep -Seconds $RESTART_DELAY
    } else {
        Write-Host "Program exited normally (Exit Code: $exit_code). No restart."
        exit 0
    }
}
