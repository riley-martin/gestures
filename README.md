# Gestures
## About
This is a program for intercepting touchpad gestures and executing commands based on them.
Unlike some alternatives, it directly uses the libinput api rather than parsing the output
of `libinput debug-events`.

## Features
`gestures` is able to handle libinput swipe events; not only vertical and horizontal but diagonal
as well.
- [x] Handle libinput events
  - [x] Swipe events; vertical, horizontal and diagonal
  - [ ] Pinch events
  - [ ] Hold events
- [ ] Config file, currently configuration is compiled in; this is top priority

## Installation
### Manual installation
- Clone the repo
  - `git clone https://github.com/riley-martin/gestures`

- Build
  - `cargo build --release`

- Copy `gestures/target/release/gestures` to a convenient place and execute it

## Alternatives
Here are some alternatives that may suit your use case better, as well as the reasons I don't use them.

- [libinput-gestures](https://github.com/bulletmark/libinput-gestures)  
Parses `libinput debug-events` rather than using libinput api, which is less memory and cpu efficient
- [gebaar](https://github.com/Coffee2CodeNL/gebaar-libinput)
Not maintained, only supports swipe gestures
- [gebaar-libinput-fork](https://github.com/osleg/gebaar-libinput-fork)
Fork of gebaar which supports other gestures, but is also not actively developed
- [fusuma](https://github.com/iberianpig/fusuma)
Also parses `libinput debug-events` output


