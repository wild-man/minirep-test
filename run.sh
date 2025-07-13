#!/usr/bin/env bash
killall miningrep
cargo build --release
cargo run --release &

sleep 5

for i in {1..100}
do
    timestamp=$(date +%s)
    hashrate=$(shuf -i 1000-25000 -n 1)
    worker_id=$(shuf -i 1-5 -n 1)
    temperature=$(shuf -i 1-100 -n 1)
    data=$(echo {\"worker_id\": \"worker-$worker_id\", \"pool\": \"us\", \"hashrate\": $hashrate, \"temperature\": $temperature, \"timestamp\": $timestamp})

    echo "inserting test data in pool (us): $data"
    curl -d "$data" -H "Content-Type: application/json"  http://localhost:8000/report

    timestamp=$(date +%s)
    hashrate=$(shuf -i 10000-65000 -n 1)
    worker_id=$(shuf -i 1-5 -n 1)
    temperature=$(shuf -i 200-800 -n 1)
    data=$(echo {\"worker_id\": \"worker-$worker_id\", \"pool\": \"eu\", \"hashrate\": $hashrate, \"temperature\": $temperature, \"timestamp\": $timestamp})

    echo "inserting test data in pool (eu): $data"
    curl -d "$data" -H "Content-Type: application/json"  http://localhost:8000/report

done

#open:
curl -s http://localhost:8000/stats |jq
killall miningrep
