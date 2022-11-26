# Gestures configuration
## Location
`$HOME/.config/gestures.conf`, `$HOME/.config/gestures/gestures.conf` and `$HOME/.gestures.conf`
are the configuration locations. They are read in that order, stopping whenever the first one is
encountered.
## Format
The configuration format is based on s-expressions.
```
(
  ;; device specifies which touchpad device to use. If left empty, selection is automatic.
  ;; Currently HAS NO EFFECT
  (device)
  ;; list of gestures. Available options are `swipe`, `pinch`, `hold` and `rotate`.
  ;; Only `swipe` and `pinch` are currently implemented, others are ignored.
  ;;
  ;; All fields shown are required
  (gestures
    (swipe
      ;; `direction`: can be N, S, E, W, NE, NW, SE or SW
      (direction . N)
      ;; `fingers`: basically can be 3 or 4, because less than three libinput does not recognize
      ;; as a gesture, and AFAICT more than four are not counted
      (fingers . 4)
      ;; `action`: command to execute. Anything that works with `sh -c` should work here.
      (action . "rofi -show drun")
    )
    (pinch
      ;; currently does nothing, may be used in the future
      (scale . 1.0)
      ;; same as above
      (fingers . 3)
      ;; `direction`: in or out
      (direction . in)
      ;; same as above
      (action . "killall rofi")
    )
  )
)
  
```
