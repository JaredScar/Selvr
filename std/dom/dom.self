// std/dom/dom.self
// DOM and Web API wrappers — idiomatic Selvr APIs, auto-targeting aware.
//
// All functions in this module are scored as JS by the targeting pass
// (they call document.*, window.*, or addEventListener).
// They will NEVER be routed to WASM; #[js] is implied by their content.

// ── Element selection ────────────────────────────────────────────────────────

/// Return the first element matching `selector`, or null.
#[js]
export fn query(selector: string): Element {
    return document.querySelector(selector);
}

/// Return all elements matching `selector` as an array.
#[js]
export fn query_all(selector: string): Element[] {
    return Array.from(document.querySelectorAll(selector));
}

/// Return the element with the given id, or null.
#[js]
export fn by_id(id: string): Element {
    return document.getElementById(id);
}

// ── Element mutation ─────────────────────────────────────────────────────────

/// Set the text content of an element.
#[js]
export fn set_text(el: Element, text: string): void {
    el.textContent = text;
}

/// Set the inner HTML of an element.
#[js]
export fn set_html(el: Element, html: string): void {
    el.innerHTML = html;
}

/// Get the text content of an element.
#[js]
export fn get_text(el: Element): string {
    return el.textContent;
}

/// Set an attribute on an element.
#[js]
export fn set_attr(el: Element, name: string, value: string): void {
    el.setAttribute(name, value);
}

/// Get an attribute from an element. Returns empty string if not set.
#[js]
export fn get_attr(el: Element, name: string): string {
    return el.getAttribute(name);
}

/// Remove an attribute from an element.
#[js]
export fn remove_attr(el: Element, name: string): void {
    el.removeAttribute(name);
}

/// Add a CSS class to an element.
#[js]
export fn add_class(el: Element, cls: string): void {
    el.classList.add(cls);
}

/// Remove a CSS class from an element.
#[js]
export fn remove_class(el: Element, cls: string): void {
    el.classList.remove(cls);
}

/// Toggle a CSS class on an element.
#[js]
export fn toggle_class(el: Element, cls: string): void {
    el.classList.toggle(cls);
}

/// Return true if the element has the given CSS class.
#[js]
export fn has_class(el: Element, cls: string): boolean {
    return el.classList.contains(cls);
}

// ── Element creation ─────────────────────────────────────────────────────────

/// Create a new element with the given tag name.
#[js]
export fn create(tag: string): Element {
    return document.createElement(tag);
}

/// Append a child element to a parent.
#[js]
export fn append_child(parent: Element, child: Element): void {
    parent.appendChild(child);
}

/// Remove a child element from its parent.
#[js]
export fn remove_child(parent: Element, child: Element): void {
    parent.removeChild(child);
}

/// Remove an element from the DOM.
#[js]
export fn remove_element(el: Element): void {
    el.remove();
}

// ── Events ───────────────────────────────────────────────────────────────────

/// Add an event listener to an element.
#[js]
export fn on(el: Element, event: string, handler: EventHandler): void {
    el.addEventListener(event, handler);
}

/// Remove an event listener from an element.
#[js]
export fn off(el: Element, event: string, handler: EventHandler): void {
    el.removeEventListener(event, handler);
}

/// Dispatch a custom event on an element.
#[js]
export fn dispatch(el: Element, event_name: string): void {
    el.dispatchEvent(new CustomEvent(event_name));
}

// ── Window ───────────────────────────────────────────────────────────────────

/// Schedule `fn` to run after `ms` milliseconds.
#[js]
export fn set_timeout(ms: i32): void {
    window.setTimeout(ms);
}

/// Schedule `fn` to run on the next animation frame.
#[js]
export fn request_animation_frame(): void {
    window.requestAnimationFrame();
}

/// Return the current timestamp in milliseconds (high-resolution).
#[js]
export fn now(): f64 {
    return performance.now();
}

/// Return the current URL as a string.
#[js]
export fn current_url(): string {
    return window.location.href;
}

/// Scroll the page to the top.
#[js]
export fn scroll_to_top(): void {
    window.scrollTo(0, 0);
}

// ── Forms ────────────────────────────────────────────────────────────────────

/// Return the current value of an input element.
#[js]
export fn input_value(el: Element): string {
    return el.value;
}

/// Set the value of an input element.
#[js]
export fn set_input_value(el: Element, value: string): void {
    el.value = value;
}

/// Return true if a checkbox is checked.
#[js]
export fn is_checked(el: Element): boolean {
    return el.checked;
}
