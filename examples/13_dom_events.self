// 13 — DOM manipulation and events
// DOM APIs feel familiar to TypeScript/JavaScript developers.
// Event handlers are typed closures — no more `event: any`.

import { query, create } from "std/web/dom";
import { MouseEvent } from "std/web/events";

struct Counter {
    value: i32;
    element: Element;
}

impl Counter {
    fn new(selector: string): Option<Counter> {
        const element = query(selector)?;
        return Some(Counter { value: 0, element });
    }

    fn render(): void {
        this.element.setText(`${this.value}`);
    }

    fn increment(): void {
        this.value += 1;
        this.render();
    }

    fn decrement(): void {
        this.value -= 1;
        this.render();
    }

    fn reset(): void {
        this.value = 0;
        this.render();
    }
}

async fn main(): void {
    let counter = Counter.new("#counter-value").unwrap();
    counter.render();

    const incBtn = query("#increment").unwrap();
    const decBtn = query("#decrement").unwrap();
    const rstBtn = query("#reset").unwrap();

    // Event listeners use typed arrow functions — no more casting from `any`
    incBtn.on("click", (_: MouseEvent) => { counter.increment(); });
    decBtn.on("click", (_: MouseEvent) => { counter.decrement(); });
    rstBtn.on("click", (_: MouseEvent) => { counter.reset(); });
}
