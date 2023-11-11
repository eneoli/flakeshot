# What is `flakeshot`?

`flakeshot` is a screenshot-tool for unix-systems which runs natively on `wayland` *and* `x11`.

# Motivation
We all know some screenshot tools like [flameshot] (for X11) or [grim] (for wayland) but they
were often only either for X11 *or* wayland. `flakeshot` should close this gap by being a screenshot
tool which should run on X11 as good as on wayland.

# Status
`flakeshot` is still under heavy development and not considered useable.

# License
See [LICENSE.txt].

# Credits
- [`screenshots-rs`] for x11-backend-screenshot-code because otherwise it would have been
  a pain to find out how to get a screenshot with x11.

[flameshot]: https://github.com/flameshot-org/flameshot
[grim]: https://sr.ht/~emersion/grim/
[LICENSE.txt]: ./LICENSE.txt
[`screenshots-rs`]: https://github.com/nashaofu/screenshots-rs