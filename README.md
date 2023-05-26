# bardiel

> **Warning**
> WIP!

This project is an opportunity for me to learn [tokio](https://github.com/tokio-rs/tokio) and how proxies work by making one.

Followed what other projects like [Velocity](https://github.com/PaperMC/Velocity) and [Infrared](https://github.com/haveachin/infrared) do and the [wiki.vg](https://wiki.vg/Protocol) fantastic documentation on the Minecraft protocol.

Since this is mostly a fun project for me, I cannot stress enough that this is **not** a replacement for anything.

## Todo:

-   Handle more packets from the login flow

## Tested on:

-   Fabric server 1.19.2/4, offline mode, network-compression-threshold -1

## Current state:

Using very scientific methods of testing, which translates to just using [mc-bots](https://github.com/crpmax/mc-bots) to create 500 fake players (not using the -x or -m flags), the proxy hovers around 8% cpu usage on a Ryzen 7 2700 and around 10mb of memory.

Is this good? No idea, even if it is, this proxy currently doesn't have anywhere near the amount of features present in other proxies as it just forwards packets, any comparison and benchmarks are pointless.

## Should I use-

Use [Velocity](https://github.com/PaperMC/Velocity).
