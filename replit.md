## Overview
A Minecraft clone written in Rust using `wgpu` and `winit`.

## Recent Changes
- Fixed compilation error: Added `stasis` field to `Player` struct in `src/player.rs`.
- Made audio system optional to prevent crashes on systems without audio devices.
- Configured VNC workflow for graphical output.
- Installed system dependencies (X11, Alsa, Vulkan, etc).

## Project Architecture
- **Renderer**: `src/renderer.rs` using `wgpu`.
- **World**: `src/world.rs` for chunk and block management.
- **Player**: `src/player.rs` for movement and inventory.
- **Network**: `src/network.rs` for multiplayer support.
