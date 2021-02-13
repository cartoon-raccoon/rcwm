#!/bin/bash

#! please don't use this, it doesn't work

cargo build

Xephyr -br -ac -reset -screen 800x600 :1 &
DISPLAY=:1 awesome