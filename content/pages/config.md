+++
title = "Configuration"
path = "config"
template = "pages.html"
draft = false
+++

Gestures uses a config format based on s-expressions. Here is a commented example.
```lisp
  (
    ; This is not implemented and is optional. If not present, the device will be automatically selected
    (device "/path/to/touchpad")
    
    ; This section is a list of gestures to intercept
    (
      ; swipe gesture
      (swipe
        ; direction: any of the eight points on a compass. Vertical and horizontal as well as diagonal
        ; gestures are supported
        (direction . N)
        ; number of fingers to use. Basically can be 3..=4 because 2 fingers is a scroll and I
        ; think libinput ignores more than 4
        (fingers . 4)
        ; the action is a command executed with `sh -c`
        (action . "rofi -show drun")
        ; repeat can be continuous or oneshot. Oneshot executes the action once after the entire gesture,
        ; continuous executes it every time `GestureSwipeEvent::Update` is recieved from libinput
        (repeat . oneshot)
      )
      ; pinch gesture
      (pinch
        ; direction can be in or out
        (direction . out)
        ; same as above, although I think a 2 finger pinch should work here
        (fingers . 4)
        ; this is currently ignored
        (scale . 0.0)
        ; same as above
        (action . "rofi -show drun")
        ; same as above
        (repeat . oneshot)
      )
      ; hold gesture
      ; shold be self-explanatory, one difference is that only oneshot hold gestures are supported,
      ; because libinput does not send a `GestureHoldEvent::Update`, only begin and end.
      (hold
        (fingers . 4)
        (action . "rofi -show drun")
      )
    )
  )
```
