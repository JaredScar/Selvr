// ERROR: .await used outside async fn
// EXPECT: AwaitOutsideAsync

fn fetchSync(url: string): string {
    const res = fetch(url).await;  // ERROR: not inside async fn
    return res;
}

fn main(): void {
    console.log(fetchSync("https://example.com"));
}
