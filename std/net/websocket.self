// std/net/websocket.self
// WebSocket client wrapper.
//
// Provides an idiomatic Selvr API over the browser's native WebSocket.
// All handlers are scored as JS (event-driven, no numeric computation).

/// Open a WebSocket connection to `url`.
/// Returns a WebSocket object; store it and pass to the other functions.
#[js]
export fn ws_connect(url: string): WebSocket {
    return new WebSocket(url);
}

/// Register a handler called when the connection opens.
#[js]
export fn on_open(ws: WebSocket, handler: EventHandler): void {
    ws.addEventListener("open", handler);
}

/// Register a handler called when a message is received.
/// The handler receives the MessageEvent; use `event.data` for the payload.
#[js]
export fn on_message(ws: WebSocket, handler: EventHandler): void {
    ws.addEventListener("message", handler);
}

/// Register a handler called when an error occurs.
#[js]
export fn on_error(ws: WebSocket, handler: EventHandler): void {
    ws.addEventListener("error", handler);
}

/// Register a handler called when the connection closes.
#[js]
export fn on_close(ws: WebSocket, handler: EventHandler): void {
    ws.addEventListener("close", handler);
}

/// Send a text message over the WebSocket.
#[js]
export fn send_text(ws: WebSocket, msg: string): void {
    ws.send(msg);
}

/// Close the WebSocket connection.
#[js]
export fn ws_close(ws: WebSocket): void {
    ws.close();
}

/// Return true if the WebSocket is open (readyState === 1).
#[js]
export fn is_open(ws: WebSocket): boolean {
    return ws.readyState === 1;
}
