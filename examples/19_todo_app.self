// 19 — TodoMVC
// A minimal but complete todo app in SELVR.
// For TypeScript developers: notice how state management is just a struct with methods.

import { query, create } from "std/web/dom";
import { KeyboardEvent }  from "std/web/events";

enum Filter { All, Active, Completed }

struct Todo {
    id: i32;
    text: string;
    completed: boolean;
}

struct App {
    todos: Todo[];
    nextId: i32;
    filter: Filter;
}

impl App {
    fn new(): App {
        return App { todos: [], nextId: 1, filter: Filter.All };
    }

    fn add(text: string): void {
        if text.trim().length === 0 { return; }
        this.todos.push(Todo { id: this.nextId, text, completed: false });
        this.nextId += 1;
    }

    fn toggle(id: i32): void {
        for todo in this.todos {
            if todo.id === id {
                todo.completed = !todo.completed;
            }
        }
    }

    fn remove(id: i32): void {
        this.todos = this.todos.filter((t) => t.id !== id);
    }

    fn clearCompleted(): void {
        this.todos = this.todos.filter((t) => !t.completed);
    }

    fn visible(): Todo[] {
        return match this.filter {
            Filter.All       => this.todos,
            Filter.Active    => this.todos.filter((t) => !t.completed),
            Filter.Completed => this.todos.filter((t) => t.completed),
        };
    }

    fn remaining(): i32 {
        return this.todos.filter((t) => !t.completed).length as i32;
    }

    fn render(): void {
        const list = query("#todo-list").unwrap();
        list.clear();

        for todo in this.visible() {
            const item = create("li");
            if todo.completed { item.addClass("completed"); }

            const checkbox = create("input");
            checkbox.setAttribute("type", "checkbox");
            if todo.completed { checkbox.setAttribute("checked", "true"); }
            const id = todo.id;
            checkbox.on("change", (_) => { this.toggle(id); this.render(); });

            const label = create("label");
            label.setText(todo.text);

            const delBtn = create("button");
            delBtn.setText("✕");
            delBtn.on("click", (_) => { this.remove(id); this.render(); });

            item.append(checkbox, label, delBtn);
            list.append(item);
        }

        query("#items-left").unwrap()
            .setText(`${this.remaining()} items left`);
    }
}

async fn main(): void {
    let app = App.new();
    app.add("Learn SELVR");
    app.add("Build something cool");
    app.add("Tell a friend");

    const input = query<InputElement>("#new-todo").unwrap();

    input.on("keydown", (e: KeyboardEvent) => {
        if e.key === "Enter" {
            app.add(input.value);
            input.value = "";
            app.render();
        }
    });

    query("#filter-all").unwrap().on("click",
        (_) => { app.filter = Filter.All;       app.render(); });
    query("#filter-active").unwrap().on("click",
        (_) => { app.filter = Filter.Active;    app.render(); });
    query("#filter-completed").unwrap().on("click",
        (_) => { app.filter = Filter.Completed; app.render(); });
    query("#clear-completed").unwrap().on("click",
        (_) => { app.clearCompleted(); app.render(); });

    app.render();
}
