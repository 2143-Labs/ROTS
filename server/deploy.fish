#!/usr/bin/env fish
set my_pid %self
echo "Our PID is $my_pid"

function my_signal_handler --on-signal SIGINT
    echo Got SIGINT signal!
    unlaunch
    exit 0
end

function last_pid
    pgrep -f "ROTS/target/release/server"
end

function launch
    cargo r --release &
    echo "Starting... "(last_pid)
end

function unlaunch
    #kill %ROTS/target/release/server
    set pid (last_pid)
    echo "killing $pid"
    kill $pid
end


# Main loop: keep exactly 1 istance of server alive, restart it on git pull
launch
while true do;
    echo (last_pid)
    git fetch
    if not test (git rev-parse HEAD) = (git rev-parse @{u});
        unlaunch
        git pull
        launch
    else
        sleep 10
    end
end
