// 09 — Traits
// Traits are like TypeScript interfaces, but they can have default method bodies
// and be used as constraints on generic type parameters.

trait Animal {
    fn name(): string;
    fn sound(): string;

    // Default method — can be overridden by implementations
    fn describe(): string {
        return `The ${this.name()} says ${this.sound()}.`;
    }
}

struct Dog { name: string; }
struct Cat { name: string; }
struct Parrot { name: string; phrase: string; }

impl Animal for Dog {
    fn name(): string  { return this.name; }
    fn sound(): string { return "woof"; }
}

impl Animal for Cat {
    fn name(): string  { return this.name; }
    fn sound(): string { return "meow"; }
}

impl Animal for Parrot {
    fn name(): string  { return this.name; }
    fn sound(): string { return this.phrase; }

    // Override the default description
    fn describe(): string {
        return `The parrot ${this.name} squawks: "${this.phrase}!"`;
    }
}

// Generic function constrained by the Animal trait — like a TypeScript generic
// with an interface constraint: <A extends Animal>(animal: A) => void
fn makeNoise<A: Animal>(animal: A): void {
    console.log(animal.describe());
}

fn main(): void {
    const dog    = Dog    { name: "Rex" };
    const cat    = Cat    { name: "Whiskers" };
    const parrot = Parrot { name: "Polly", phrase: "pretty bird" };

    makeNoise(dog);    // The Rex says woof.
    makeNoise(cat);    // The Whiskers says meow.
    makeNoise(parrot); // The parrot Polly squawks: "pretty bird!"
}
