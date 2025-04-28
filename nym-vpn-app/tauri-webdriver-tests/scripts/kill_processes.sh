#!/bin/bash
echo "Checking for processes using ports 1420 and 4444..."

# STRANGE VS CODE BUG - LINUX - PUTTING THIS HERE TEMPORARILY
# unset GTK_PATH

kill_processes_on_port() {
    local port=$1
    local pids
    
    pids=$(lsof -i :"${port}" -t 2>/dev/null)
    
    if [ -z "$pids" ]; then
        echo "No processes found on port ${port}"
        return 0
    else
        echo "Found processes on port ${port}: $pids"
        
        echo "Attempting graceful shutdown with SIGTERM..."
        for pid in $pids; do
            echo "Sending SIGTERM to process $pid"
            kill "$pid" 2>/dev/null || true
        done
        
        sleep 1
        
        remaining_pids=$(lsof -i :"${port}" -t 2>/dev/null)
        
        if [ -n "$remaining_pids" ]; then
            echo "Some processes didn't terminate gracefully, using SIGKILL..."
            for pid in $remaining_pids; do
                echo "Sending SIGKILL to process $pid"
                kill -9 $pid 2>/dev/null || true
            done
        fi
        
        final_check=$(lsof -i :"${port}" -t 2>/dev/null)
        if [ -n "$final_check" ]; then
            echo "WARNING: Could not kill all processes on port ${port}: $final_check"
            return 1
        else
            echo "Successfully killed all processes on port ${port}"
            return 0
        fi
    fi
}

echo "Checking port 1420 (Vite development server)..."
kill_processes_on_port 1420
vite_result=$?

echo "Checking port 4444 (geckodriver)..."
kill_processes_on_port 4444
gecko_result=$?

echo "Checking for remaining geckodriver processes by name..."
gecko_pids=$(pgrep -f "geckodriver" 2>/dev/null)
if [ -n "$gecko_pids" ]; then
    echo "Found additional geckodriver processes: $gecko_pids"
    for pid in $gecko_pids; do
        echo "Killing geckodriver process $pid"
        kill -9 "$pid" 2>/dev/null || true
    done
else
    echo "No additional geckodriver processes found"
fi

browser_pids=$(pgrep -f "firefox.*marionette" 2>/dev/null)
if [ -n "$browser_pids" ]; then
    echo "Found Firefox instances that might be related to WebDriver: $browser_pids"
    echo "You may want to review these processes"
fi

echo "=== Cleanup Summary ==="
if [ $vite_result -eq 0 ] && [ $gecko_result -eq 0 ]; then
    echo "✅ All ports successfully cleared or were already free"
    exit 0
else
    echo "There were issues clearing some ports"
    exit 1
fi