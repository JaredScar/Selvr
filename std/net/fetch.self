// std/net/fetch.self
// HTTP client wrappers — idiomatic Selvr API over the browser Fetch API.
//
// All functions are scored as JS (async + fetch calls).
// Phase 2 WASM bridge: response body parsing (JSON→struct) may be routed
// to a WASM parser in the future for large payloads.

/// Perform a GET request and return the response body as a string.
/// Use `await` when calling this function.
#[js]
export async fn get(url: string): string {
    let response = await fetch(url);
    let text = await response.text();
    return text;
}

/// Perform a GET request and parse the response as JSON.
/// Returns a dynamic value — use pattern matching to destructure.
#[js]
export async fn get_json(url: string): any {
    let response = await fetch(url);
    let data = await response.json();
    return data;
}

/// Perform a POST request with a JSON body.
#[js]
export async fn post_json(url: string, body: string): string {
    let response = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: body,
    });
    let text = await response.text();
    return text;
}

/// Perform a PUT request with a JSON body.
#[js]
export async fn put_json(url: string, body: string): string {
    let response = await fetch(url, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: body,
    });
    return await response.text();
}

/// Perform a DELETE request.
#[js]
export async fn delete_req(url: string): boolean {
    let response = await fetch(url, { method: "DELETE" });
    return response.ok;
}

/// Perform a request with custom options; returns the Response object.
#[js]
export async fn request(url: string, options: any): any {
    return await fetch(url, options);
}

/// Check if a response was successful (status 200–299).
#[js]
export fn is_ok_status(status: i32): boolean {
    return status >= 200 && status < 300;
}
