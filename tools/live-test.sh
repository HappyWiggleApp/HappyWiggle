#!/usr/bin/env bash
# End-to-end smoke test on Linux: start the real app on a virtual display with
# a timetable that is active right now, and check that the real pointer moves.
# Run: xvfb-run -a tools/live-test.sh
set -u
BIN="$(dirname "$0")/../target/debug/happywiggle"
export XDG_CONFIG_HOME="$(mktemp -d)"
mkdir -p "$XDG_CONFIG_HOME/app.happywiggle"
cat > "$XDG_CONFIG_HOME/app.happywiggle/settings.json" <<JSON
{ "mode": "wiggle", "wiggleMinS": 1, "wiggleMaxS": 2, "keepAwake": false, "startHidden": true,
  "schedule": { "enabled": true, "blocks": [ { "days": [true,true,true,true,true,true,true],
    "startMin": 0, "endMin": 1440, "startJitter": {"kind":"exact","minutes":0}, "endJitter": {"kind":"exact","minutes":0}, "break": null, "mode": null } ] } }
JSON
xdotool mousemove 400 300
"$BIN" > "$XDG_CONFIG_HOME/log.txt" 2>&1 &
PID=$!
positions=""
for i in $(seq 1 400); do
  sleep 0.03
  positions="$positions $(xdotool getmouselocation | cut -d' ' -f1,2 | tr ' ' ',')"
done
alive=no; kill -0 $PID 2>/dev/null && alive=yes
kill $PID 2>/dev/null; wait $PID 2>/dev/null
echo $positions | tr " " "\n" | uniq -c | head -40 > "$XDG_CONFIG_HOME/timeline.txt"; cat "$XDG_CONFIG_HOME/timeline.txt" | head -${SHOW:-0}
distinct=$(echo $positions | tr ' ' '\n' | sort -u | wc -l)
final=$(xdotool getmouselocation | cut -d' ' -f1,2)
echo "app alive at the end: $alive"
echo "distinct pointer positions seen: $distinct"
echo "pointer at the end: $final (started at x:400 y:300)"
grep -i "panic\|error" "$XDG_CONFIG_HOME/log.txt" | head -5
ls "$XDG_CONFIG_HOME/app.happywiggle"
[ "$alive" = yes ] && [ "$distinct" -gt 10 ] && echo "LIVE TEST OK" || { echo "LIVE TEST FAILED"; tail -20 "$XDG_CONFIG_HOME/log.txt"; exit 1; }
