// 18 — Structured error handling
// SELVR uses Result<T, E> instead of throw/catch.
// Errors are values — they compose, they're typed, and the compiler enforces handling.
// TypeScript equivalent: never throwing Error objects, always returning typed results.

enum AppError {
    Network(string);
    Parse   { input: string; reason: string };
    NotFound { id: i32 };
    Database(string);
}

impl AppError {
    fn toString(): string {
        return match this {
            AppError.Network(msg)              => `Network error: ${msg}`,
            AppError.Parse { input, reason }   => `Parse error on "${input}": ${reason}`,
            AppError.NotFound { id }           => `Not found: id=${id}`,
            AppError.Database(msg)             => `Database error: ${msg}`,
        };
    }
}

fn getUserName(id: i32): Result<string, AppError> {
    return match id {
        0  => Err(AppError.NotFound { id: 0 }),
        1  => Ok("Alice"),
        2  => Ok("Bob"),
        _  => Err(AppError.Database("connection refused")),
    };
}

fn parseUserId(input: string): Result<i32, AppError> {
    return input.parseInt().mapErr((_) => AppError.Parse {
        input,
        reason: "not a valid integer",
    });
}

// ? chains errors through the call stack — like async/await for errors
fn handleRequest(idStr: string): Result<string, AppError> {
    const id   = parseUserId(idStr)?;
    const name = getUserName(id)?;
    return Ok(`Hello, ${name}!`);
}

fn main(): void {
    const testCases = ["1", "2", "0", "99", "abc"];

    for input in testCases {
        match handleRequest(input) {
            Ok(msg) => console.log(`[OK]  ${msg}`),
            Err(e)  => console.log(`[ERR] ${e.toString()}`),
        }
    }
}
