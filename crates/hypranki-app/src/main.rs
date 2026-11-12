use hypranki_app::entry_point;

#[tokio::main]
async fn main() {
    entry_point::entry_point().await.unwrap();
}
