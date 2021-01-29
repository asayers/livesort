* [x] Get something working.
* [x] If the output exceeds the height of the terminal, and the user
      scrolls back, things are pretty messed up.
* [ ] If the output exceeds the width of the terminal, we don't clear
      enough lines.
* [x] We could at least try to detect a non-interactive output and just behave
      like regular `sort`.
* [ ] Currently we redraw the entire screen on every new line.  It should be
      possible to be much more efficient by simply moving the cursor to the
      correct location and inserting a line.
* [ ] If the user shrinks the height of the terminal while livesort is running,
      some junk will be left in the scrollback buffer.  (There's no issue
      with increasing the height.)
