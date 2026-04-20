// 22 — WebSocket chat client
// For TypeScript developers: the WebSocket API looks exactly like the browser's
// native WebSocket, but with typed messages and no callback hell.

import { query }                   from "std/web/dom";
import { WebSocket, Message }      from "std/web/socket";
import { KeyboardEvent }           from "std/web/events";

struct ChatApp {
    socket: WebSocket;
    username: string;
    messages: string[];
}

impl ChatApp {
    async fn connect(url: string, username: string): Result<ChatApp, string> {
        const socket = WebSocket.connect(url).await
            .mapErr((e) => `WebSocket connect failed: ${e}`)?;
        return Ok(ChatApp { socket, username, messages: [] });
    }

    async fn sendMessage(text: string): Result<void, string> {
        if text.trim().length === 0 { return Ok(()); }
        const payload = `${this.username}: ${text}`;
        return this.socket.send(payload).await
            .mapErr((e) => `Send failed: ${e}`);
    }

    fn onMessage(msg: Message): void {
        this.messages.push(msg.data);
        this.render();
    }

    fn render(): void {
        const list = query("#messages").unwrap();
        list.clear();
        for msg in this.messages {
            const item = create("li");
            item.setText(msg);
            list.append(item);
        }
        list.scrollTop = list.scrollHeight;
    }
}

async fn main(): void {
    const usernameEl = query<InputElement>("#username").unwrap();
    const connectBtn = query("#connect").unwrap();

    connectBtn.on("click", async (_) => {
        const username = usernameEl.value.trim();
        if username.length === 0 {
            query("#status").unwrap().setText("Please enter a username.");
            return;
        }

        query("#status").unwrap().setText("Connecting...");

        const app = match ChatApp.connect("wss://echo.websocket.org", username).await {
            Ok(a)  => a,
            Err(e) => {
                query("#status").unwrap().setText(`Error: ${e}`);
                return;
            }
        };

        query("#status").unwrap().setText("Connected!");
        query("#chat-ui").unwrap().removeClass("hidden");

        app.socket.onMessage((msg) => app.onMessage(msg));

        const messageInput = query<InputElement>("#message-input").unwrap();
        messageInput.on("keydown", async (e: KeyboardEvent) => {
            if e.key === "Enter" {
                const _ = app.sendMessage(messageInput.value).await;
                messageInput.value = "";
            }
        });
    });
}
