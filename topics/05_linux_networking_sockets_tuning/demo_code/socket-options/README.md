# Socket Options Demo

Shows how to apply TCP_NODELAY etc. on a raw socket.

In real life you configure these options through your WebSocket / TCP library (tungstenite, tokio-tungstenite, etc.) or by accessing the underlying stream.

The important lesson is **know what your library is doing under the hood**.
