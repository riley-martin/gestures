# Gestures configuration
## Location
`$HOME/.config/gestures.conf`, `$HOME/.config/gestures/gestures.conf` and `$HOME/.gestures.conf`
are the configuration locations. They are read in that order, stopping whenever the first one is
encountered.
## Format
The configuration format is based on s-expressions.
```lisp
(
  ; device specifies which touchpad device to use. If left empty, selection is automatic.
  ; Currently HAS NO EFFECT
  ; (device)
  ; list of gestures. Available options are `swipe`, `pinch`, `hold` and `rotate`.
  ; Only `swipe` and `pinch` are currently implemented, others are ignored.
  ;
  ; All fields shown are required
  (gestures
    (swipe
      ; `direction`: can be N, S, E, W, NE, NW, SE, SW or any
      (direction . N)
      ; `fingers`: basically can be 3 or 4, because less than three libinput does not recognize
      ; as a gesture, and AFAICT more than four are not counted
      (fingers . 4)
      ; `action`: command to execute on update event. Anything that works with `sh -c` should work here.
      (update . (""))
      ; `start`: command to execute on start event
      (start . (""))
      ; `end`: command to execute on end event
      (end . ("rofi -show drun"))
    )
    (pinch
      ; same as above
      (fingers . 3)
      ; `direction`: in or out or any
      (direction . in)
      ; same as above
      (update . (""))
      ; same as above
      (start . (""))
      ; same as above
      (end . ("killall rofi"))
    )
    ; hold action
    ; note that only oneshot is supported here
    (hold
      (fingers . 4)
      (action . "rofi -show drun")
    )
  )
)
  
```
