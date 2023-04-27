use mini_redis::{Result, client};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    let result =client.set("hello", "jade".into()).await?;
    println!("Set from server={:?}", result);

    let result = client.get("hello").await?;
    
    println!("Get from server={:?}", result);

    Ok(())
}
