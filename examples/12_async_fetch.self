// 12 — Async / await and fetch
// async fn and .await work exactly like TypeScript's async/await.
// Errors are typed with Result<T, E> instead of thrown exceptions.

import { fetch } from "std/web";

struct Post {
    id: i32;
    title: string;
    body: string;
}

struct Comment {
    postId: i32;
    name: string;
    body: string;
}

async fn fetchPost(id: i32): Result<Post, FetchError> {
    const url = `https://jsonplaceholder.typicode.com/posts/${id}`;
    const post = fetch(url).await?.json<Post>().await?;
    return Ok(post);
}

async fn fetchComments(postId: i32): Result<Comment[], FetchError> {
    const url = `https://jsonplaceholder.typicode.com/posts/${postId}/comments`;
    const comments = fetch(url).await?.json<Comment[]>().await?;
    return Ok(comments);
}

// Fetch both concurrently — like Promise.all in TypeScript
async fn fetchPostWithComments(id: i32): Result<[Post, Comment[]], FetchError> {
    const [post, comments] = join!(
        fetchPost(id),
        fetchComments(id),
    ).await;
    return Ok([post?, comments?]);
}

async fn main(): void {
    match fetchPostWithComments(1).await {
        Ok([post, comments]) => {
            console.log(`Post: ${post.title}`);
            console.log(`Comments: ${comments.length}`);
            for comment in comments {
                console.log(`  - ${comment.name}: ${comment.body}`);
            }
        }
        Err(e) => {
            console.log(`Failed to fetch: ${e}`);
        }
    }
}
