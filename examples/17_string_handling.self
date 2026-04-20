// 17 — Strings
// string methods follow TypeScript/JavaScript naming conventions.
// Template literals use the same backtick syntax as TypeScript.

fn isPalindrome(s: string): boolean {
    const chars = s.split("");
    const n = chars.length;
    for i in 0..n / 2 {
        if chars[i] !== chars[n - 1 - i] {
            return false;
        }
    }
    return true;
}

fn titleCase(s: string): string {
    return s.split(" ")
            .map((word) => {
                if word.length === 0 { return word; }
                return word[0].toUpperCase() + word.slice(1).toLowerCase();
            })
            .join(" ");
}

fn countVowels(s: string): i32 {
    return s.split("").filter((c) => "aeiouAEIOU".includes(c)).length as i32;
}

fn main(): void {
    // Template literals — identical backtick syntax to TypeScript
    const lang = "SELVR";
    const version = 1;
    console.log(`${lang} version ${version}`);

    // String methods — same names as JavaScript/TypeScript
    const greeting = "Hello, world!";
    console.log(greeting.length);                    // 13
    console.log(greeting.toUpperCase());             // HELLO, WORLD!
    console.log(greeting.includes("world"));         // true
    console.log(greeting.replace("world", "SELVR")); // Hello, SELVR!

    // Split and join — identical to TypeScript
    const csv = "Alice,Bob,Carol,Dave";
    const names = csv.split(",");
    console.log(names.length);          // 4
    console.log(names.join(" & "));     // Alice & Bob & Carol & Dave

    // Palindrome check
    const words = ["racecar", "hello", "level", "world", "madam"];
    for word in words {
        console.log(`${word}: ${isPalindrome(word)}`);
    }

    // Title case
    console.log(titleCase("the quick brown fox")); // The Quick Brown Fox

    // Count vowels
    console.log(countVowels("Hello, world!"));  // 3

    // Unicode — length counts code units, not characters
    const emoji = "Hello 🌍!";
    console.log(emoji.length);       // 9 code units
    console.log(emoji.charCount());  // 9 visible characters (SELVR-specific)
}
