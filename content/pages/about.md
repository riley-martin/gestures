+++
title = "About"
path = "about"
template = "pages.html"
draft = false
+++

Gestures is a program for recieving touch gestures from libinput and executing user-defined
commands based on them. Because performance is important, it is written in rust and uses the
libinput rust API directly rather than parsing the output of `libinput debug-events`.  

The configuration is based on S-expressions, read more about it [here](/config).