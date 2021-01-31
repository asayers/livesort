* [x] Get something working.
* [x] If the output exceeds the height of the terminal, and the user
      scrolls back, things are pretty messed up.
* [x] If the output exceeds the width of the terminal, we don't clear
      enough lines.
* [x] If the terminal width increases, so that a line which previously
      wrapped no longer wraps, we clear too many lines in the next redraw.
* [x] We could at least try to detect a non-interactive output and just behave
      like regular `sort`.
* [ ] Currently we redraw the entire screen on every new line.  It should be
      possible to be much more efficient by simply moving the cursor to the
      correct location and inserting a line.
* [ ] If the user shrinks the height of the terminal while livesort is running,
      some junk will be left in the scrollback buffer.  (There's no issue
      with increasing the height.)  We could fix this by using the alternate
      screen for live preview and then switching back to the regular screen
      for the final dump.
* [x] Add --uniq --count mode
* [x] Add sort by frequency mode
* [ ] Redraw when the terminal geometry changes, even if no new data comes in
* [ ] Add a "...and <N> more lines" when truncating?
