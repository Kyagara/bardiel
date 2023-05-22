# Bardiel

> **Warning**
> WIP!

This project is an opportunity for me to learn [tokio](https://github.com/tokio-rs/tokio) and how proxies work by making one.

Followed what other projects like [Velocity](https://github.com/PaperMC/Velocity) and [Infrared](https://github.com/haveachin/infrared) do and followed the [wiki.vg](https://wiki.vg/Protocol) fantastic documentation on the Minecraft protocol.

Since we only deal with the Handshake and Login packets, this proxy might work with version 1.7 and above, I only used versions 1.19.2 and 1.19.4 with [Fabric](https://fabricmc.net) in my tests (online and offline mode) without any issues.

## Current state:

Using very scientific methods of testing, which translates to just using [mc-bots](https://github.com/crpmax/mc-bots) to create 500 fake players (not using the -x or -m flags), the proxy hovers around 8% cpu usage on a Ryzen 7 2700 and around 10mb of memory.

Is this good? No idea, even if it is, currently this proxy doesn't have anywhere near the amount of features present in other proxies, any comparison is pointless.

## Should I use-

Use [Velocity](https://github.com/PaperMC/Velocity).
