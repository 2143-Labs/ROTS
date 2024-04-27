#!/usr/bin/env fish
while true do;
    echo "fetching"
    git fetch
    if not test (git rev-parse HEAD) = (git rev-parse @{u});
        echo "Restarting server: got push"
        set pid (ps aux | grep ROTS/target/release/server | grep -v "rg ROTS" | choose 1)
        kill $pid
        git pull
        cargo r --release &
    end
end
