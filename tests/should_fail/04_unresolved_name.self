// ERROR: unresolved name
// EXPECT: UnresolvedName

fn main(): void {
    console.log(doesNotExist);  // ERROR: name not in scope
}
