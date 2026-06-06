use gpn_tron::{GpnTronBot, demobot::DemoBot};

#[tokio::main]
async fn main() {
    let mut bot = DemoBot::new();
    bot.start("localhost", 4000, "mytest", "testpassword")
        .await
        .unwrap();
}
