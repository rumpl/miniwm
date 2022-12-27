# miniwm

Simple minimal window manager that can slap a new window on your screen

To test it you need to build it first

```console
$ cargo build
```

In one terminal run Xephyr and the window manager

```console
$ sudo Xephyr :1 -ac -br -noreset -screen 800x600
$ DISPLAY=:1 ./target/debug/miniwm
```

And then finally run some application

```console
$ DISPLAY=:1 urxvt
```

You should see the urxvt terminal in the Xephyr screen and some logs in the first terminal (where you ran miniwm)
