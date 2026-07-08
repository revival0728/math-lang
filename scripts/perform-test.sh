#!/bin/zsh

zmodload zsh/datetime
TIMEFMT=$'%E'

sum=0

cargo build --profile release

for t in {1..100}; do
    start_time=$EPOCHREALTIME
    ./target/release/math-lang ./scripts/perform_test.mls > /dev/null
    end_time=$EPOCHREALTIME
    runtime=$(( end_time - start_time ))
    sum=$((sum+runtime))
done

echo "time: $sum"