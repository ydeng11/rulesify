Output docs/rulesify-demo.gif

Require "./target/release/rulesify"

Set Shell "bash"
Set FontSize 18
Set Width 1200
Set Height 750
Set TypingSpeed 45ms
Set PlaybackSpeed 1.0
Set Framerate 12
Set Theme "Dracula"
Set Padding 12
Set MarginFill "#1e1e2e"
Set BorderRadius 8

Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type "rm -rf /tmp/rulesify-demo"
Enter
Type "mkdir -p /tmp/rulesify-demo"
Enter
Type "cd /tmp/rulesify-demo"
Enter
Type "clear"
Enter
Show

Type "rulesify"
Sleep 500ms
Enter
Sleep 1800ms

Down
Space
Sleep 700ms
Down
Space
Sleep 1200ms
Enter
Sleep 2400ms

Right
Sleep 1400ms
Left
Sleep 1000ms
Type "s"
Sleep 1400ms

Type "t"
Sleep 1400ms
Type "debugging"
Sleep 1200ms
Right
Sleep 700ms
Space
Sleep 900ms
Enter
Sleep 1800ms

Type "/"
Type "diagnosing-bugs"
Sleep 2800ms
Enter
Sleep 700ms
Space
Sleep 1300ms
Enter
Sleep 4000ms

Type "rulesify skill list"
Sleep 500ms
Enter
Sleep 3000ms

Hide
Type "rm -rf /tmp/rulesify-demo"
Enter
